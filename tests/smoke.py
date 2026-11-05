"""Real HTTP workflow and persistence tests. No third-party Python packages."""
import concurrent.futures
import json
import os
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
from pathlib import Path

remote = len(sys.argv) > 1

root = Path(__file__).resolve().parents[1]
process = None
directory = tempfile.TemporaryDirectory()
if remote:
    base = sys.argv[1].rstrip('/')
else:
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0))
        port = sock.getsockname()[1]
    base = f'http://127.0.0.1:{port}'

def start():
    global process
    process = subprocess.Popen([str(root / os.environ.get('MCP_TEST_BINARY', 'server/target/debug/enisa-sme-mcp'))], env={**os.environ, 'HOST': '127.0.0.1', 'PORT': str(port), 'ENISA_MCP_DATA_DIR': directory.name}, stdout=subprocess.DEVNULL)
    for _ in range(100):
        try:
            urllib.request.urlopen(base + '/healthz', timeout=1).close()
            return
        except (OSError, urllib.error.URLError):
            if process.poll() is not None:
                raise RuntimeError('Server exited during startup')
            time.sleep(0.1)
    raise RuntimeError('Startup timed out')

def request(body=None, *, method='POST', extra=None):
    headers = {'Content-Type': 'application/json', 'Accept': 'application/json, text/event-stream', 'MCP-Protocol-Version': '2025-11-25'}
    headers.update(extra or {})
    req = urllib.request.Request(base + '/mcp', data=None if body is None else json.dumps(body).encode(), headers=headers, method=method)
    try:
        with urllib.request.urlopen(req, timeout=20) as response:
            raw = response.read()
            return response.status, json.loads(raw) if raw else None
    except urllib.error.HTTPError as error:
        return error.code, None

def call(name, **args):
    status, response = request({'jsonrpc': '2.0', 'id': 1, 'method': 'tools/call', 'params': {'name': name, 'arguments': args}})
    assert status == 200, (name, status)
    assert not response['result']['isError'], response
    return response['result']['structuredContent']

assessment_id = None
try:
    if not remote:
        start()
    assert request(method='GET')[0] == 405
    assert request(method='GET', extra={'Origin': 'https://evil.invalid'})[0] == 403
    assert request(method='GET')[0] == 405
    assert request({'jsonrpc': '2.0', 'id': 1, 'method': 'ping'}, extra={'MCP-Protocol-Version': 'unsupported'})[0] == 400
    status, response = request({'jsonrpc': '2.0', 'id': 1, 'method': 'initialize', 'params': {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'smoke', 'version': '1'}}})
    assert status == 200 and response['result']['protocolVersion'] == '2025-11-25'
    assert request({'jsonrpc': '2.0', 'method': 'notifications/initialized'}) == (202, None)
    status, listed = request({'jsonrpc':'2.0', 'id':2, 'method':'tools/list'})
    assert status == 200 and len(listed['result']['tools']) == 10
    assert all(t['securitySchemes'] == [{'type':'noauth'}] for t in listed['result']['tools'])
    assert len(call('get_question_bank', mode='quick')['questions']) == 12
    assessment_id = call('create_assessment', organization='Automated smoke test', mode='quick')['assessment_id']
    questions = call('get_question_bank', mode='quick')['questions']
    with concurrent.futures.ThreadPoolExecutor(max_workers=12) as pool:
        list(pool.map(lambda q: call('record_answer', assessment_id=assessment_id, question_id=q['id'], value='3', evidence='Test reference; not independently verified'), questions))
    report = call('calculate_maturity', assessment_id=assessment_id)
    assert report['overall_index_pct'] == 75.0, report
    assert not report['gaps']
    if not remote:
        process.terminate()
        process.wait(timeout=10)
        start()
        assert call('calculate_maturity', assessment_id=assessment_id)['overall_index_pct'] == 75.0
    assert any(a['assessment_id'] == assessment_id for a in call('list_assessments')['assessments'])
    call('delete_assessment', assessment_id=assessment_id)
    assert not any(a['assessment_id'] == assessment_id for a in call('list_assessments')['assessments'])
    assessment_id = None
    print('PASS: anonymous access, Origin, protocol, initialization, notifications, question bank, concurrent answers, scoring, listing, deletion' + (', restart persistence' if not remote else ''))
finally:
    if assessment_id:
        try:
            call('delete_assessment', assessment_id=assessment_id)
        except Exception:
            print('Cleanup needed for test assessment:', assessment_id, file=sys.stderr)
    if process and process.poll() is None:
        process.terminate()
        process.wait(timeout=10)
    directory.cleanup()
