"""Jupyter kernel for Grizzly.

A wrapper kernel: ipykernel speaks the Jupyter protocol, and each cell runs on
one `grizzly --cells` process, so bindings survive across cells like in the REPL.
See https://jupyter-client.readthedocs.io/en/latest/wrapperkernels.html

    python3 grizzly_kernel.py install   # registers the kernel with Jupyter
"""

import json
import os
import subprocess
import sys
import tempfile

from ipykernel.kernelbase import Kernel

# Ends each cell sent to `grizzly --cells`, and each part of its result.
END = b"\0"


class GrizzlyKernel(Kernel):
    implementation = "grizzly"
    implementation_version = "0.1.0"
    language_info = {
        "name": "grizzly",
        "mimetype": "text/x-grizzly",
        "file_extension": ".griz",
    }
    banner = "Grizzly"

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        # `grizzly` must be on PATH, e.g. after `cargo install --path .`.
        self.grizzly = subprocess.Popen(
            ["grizzly", "--cells"], stdin=subprocess.PIPE, stdout=subprocess.PIPE
        )

    def read_part(self):
        """Reads the next part of a cell's result, up to `END`."""
        part = bytearray()
        while (byte := self.grizzly.stdout.read(1)) not in (END, b""):
            part += byte
        return part.decode()

    async def do_execute(self, code, silent, store_history=True, user_expressions=None,
                         allow_stdin=False, **kwargs):
        self.grizzly.stdin.write(code.encode() + END)
        self.grizzly.stdin.flush()
        out, err = self.read_part(), self.read_part()

        if out and not silent:
            self.send_response(self.iopub_socket, "stream", {"name": "stdout", "text": out})

        # Errors are only written on failure. Like ipykernel, a failure is
        # published as an `error` output and repeated in the reply.
        if err:
            error = {"ename": "Error", "evalue": err.rstrip(), "traceback": err.splitlines()}
            if not silent:
                self.send_response(self.iopub_socket, "error", error)
            return {"status": "error", "execution_count": self.execution_count, **error}

        return {
            "status": "ok",
            "execution_count": self.execution_count,
            "payload": [],
            "user_expressions": {},
        }

    async def do_shutdown(self, restart):
        self.grizzly.kill()
        return {"status": "ok", "restart": restart}


def install():
    """Registers this kernel for the current user, run with this Python."""
    from jupyter_client.kernelspec import KernelSpecManager

    spec = {
        "argv": [sys.executable, os.path.abspath(__file__), "-f", "{connection_file}"],
        "display_name": "Grizzly",
        "language": "grizzly",
    }
    with tempfile.TemporaryDirectory() as folder:
        with open(os.path.join(folder, "kernel.json"), "w") as file:
            json.dump(spec, file, indent=2)
        path = KernelSpecManager().install_kernel_spec(folder, "grizzly", user=True)
    print(f"Installed kernel spec in {path}")


if __name__ == "__main__":
    if sys.argv[1:] == ["install"]:
        install()
    else:
        from ipykernel.kernelapp import IPKernelApp

        IPKernelApp.launch_instance(kernel_class=GrizzlyKernel)
