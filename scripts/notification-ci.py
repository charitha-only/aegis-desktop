"""Exercise real Linux notification IPC against an owned, nonvisual D-Bus service."""
import json
import os
import queue
import re
import subprocess
import sys
import tempfile
import threading
import time
import urllib.request
from pathlib import Path

if os.environ.get("GITHUB_ACTIONS") != "true" or os.environ.get("RUNNER_ENVIRONMENT") != "github-hosted" or sys.platform != "linux":
    raise RuntimeError("Notification transport fixture requires a disposable Linux hosted runner")

import dbus
import dbus.service
from dbus.mainloop.glib import DBusGMainLoop
from gi.repository import GLib

DBusGMainLoop(set_as_default=True)
bus = dbus.SessionBus()
name = dbus.service.BusName("org.freedesktop.Notifications", bus=bus, do_not_queue=True)
calls = []
errors = []
owned = []
loop = GLib.MainLoop()


class NotificationService(dbus.service.Object):
    @dbus.service.method("org.freedesktop.Notifications", in_signature="", out_signature="as")
    def GetCapabilities(self):
        return ["body", "body-markup"]

    @dbus.service.method("org.freedesktop.Notifications", in_signature="", out_signature="ssss")
    def GetServerInformation(self):
        return ("Aegis owned transport fixture", "Aegis", "1", "1.2")

    @dbus.service.method("org.freedesktop.Notifications", in_signature="susssasa{sv}i", out_signature="u")
    def Notify(self, app, replaces, icon, summary, body, actions, hints, expires):
        if str(summary) == "Rejected fixture":
            raise dbus.exceptions.DBusException("Synthetic rejection", name="org.freedesktop.Notifications.Error")
        calls.append((str(summary), str(body), int(hints.get("urgency", -1)), int(expires)))
        return len(calls)

    @dbus.service.method("org.freedesktop.Notifications", in_signature="u", out_signature="")
    def CloseNotification(self, notification_id):
        pass


service = NotificationService(name, "/org/freedesktop/Notifications")


def exercise():
    child = None
    try:
        root = Path(__file__).resolve().parents[1]
        with tempfile.TemporaryDirectory(prefix="aegis-notification-ci-") as directory:
            fixture = Path(directory)
            config = json.loads((root / "aegis.config.json").read_text(encoding="utf-8"))
            config["security"]["capabilities"]["system"]["allowNotifications"] = True
            (fixture / "aegis.config.json").write_text(json.dumps(config), encoding="utf-8")
            env = dict(os.environ, HOME=directory, XDG_DATA_HOME=str(fixture / "data"), XDG_CONFIG_HOME=str(fixture / "config"), XDG_CACHE_HOME=str(fixture / "cache"))
            child = subprocess.Popen([str(root / "src-core/target/debug/aegis-core"), "--port=0"], cwd=directory, env=env, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
            owned.append(child)
            ready = queue.Queue()

            def read_stdout():
                for line in child.stdout:
                    match = re.search(r"IPC Bridge Server actively listening on (http://127\.0\.0\.1:\d+)", line)
                    if match:
                        ready.put(match.group(1))

            threading.Thread(target=read_stdout, daemon=True).start()
            base = ready.get(timeout=15)

            def request(url, payload=None):
                body = None if payload is None else json.dumps(payload).encode("utf-8")
                req = urllib.request.Request(base + url, data=body, headers={"Content-Type": "application/json"})
                with urllib.request.urlopen(req, timeout=5) as response:
                    return json.load(response)

            token = request("/handshake")["token"]
            sequence = 0

            def call(command, payload):
                nonlocal sequence
                sequence += 1
                return request("/ipc", {"id": str(sequence), "nonce": token, "command": command, "payload": payload})

            def finish(submission):
                assert submission["success"], submission.get("error")
                assert submission["data"]["status"] == "pending"
                assert submission["data"]["deliveryConfirmed"] is False
                assert "body" not in submission["data"]
                deadline = time.monotonic() + 8
                while time.monotonic() < deadline:
                    time.sleep(0.1)
                    result = call("notify_poll", {"requestId": submission["data"]["requestId"]})
                    if not result["success"] or result["data"]["status"] != "pending":
                        return result
                raise AssertionError("Notification completion timed out")

            body = r"අ <b>literal</b> & data C:\new\folder \074tag\076"
            accepted = finish(call("notify_send", {"title": "Transport fixture", "body": body, "urgency": "critical", "timeout_ms": 5000}))
            assert accepted["success"], accepted.get("error")
            assert accepted["data"] == {"status": "providerAccepted", "deliveryConfirmed": False}
            assert calls == [("Transport fixture", r"අ &lt;b&gt;literal&lt;/b&gt; &amp; data C:\new\folder \074tag\076", 2, 5000)], calls
            rejected = finish(call("notify_send", {"title": "Rejected fixture", "body": "synthetic"}))
            assert rejected["success"] is False, rejected
            assert "data" not in rejected
            print("Real Linux notify-send/D-Bus acceptance and rejection verified; visual delivery remains unconfirmed", flush=True)
    except BaseException as error:
        errors.append(error)
    finally:
        if child is not None:
            child.terminate()
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait(timeout=5)
        GLib.idle_add(loop.quit)


def deadline():
    errors.append(TimeoutError("Notification fixture exceeded its deadline"))
    for child in owned:
        if child.poll() is None:
            child.kill()
    loop.quit()
    return False


worker = threading.Thread(target=exercise, daemon=True)
worker.start()
GLib.timeout_add_seconds(45, deadline)
loop.run()
worker.join(timeout=6)
if worker.is_alive():
    raise RuntimeError("Notification fixture worker did not stop")
if errors:
    raise errors[0]
