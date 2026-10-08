# Hardware and communication channels

## Initial channel: Ethernet TCP

ROC800 supports ROC Plus over Ethernet TCP. The configured TCP port is normally
4000; it can be changed. Sources:
[ROC800 product data sheet, communications](https://www.emerson.com/is/content/emerson/en/final-control/documents/d301155x012.pdf),
[ROCLINK configuration manual, Ethernet settings](https://www.emerson.com/is/content/emerson/en/final-control/documents/d301250x012.pdf).

For eventual physical testing, we need a ROC800-series controller with Ethernet,
its appropriate power supply, an Ethernet cable/network interface, known network
settings and configured operator access. Sensors are not necessary to validate
clock and supported parameter reads. Exact firmware and security configuration
must be recorded.

The first test channel uses a TCP listener on 127.0.0.1 with an OS-assigned port.
No hardware, Docker, privileged fixed ports or external service is required.
It tests the same framing/session/client path as the hardware connector.

## Serial channel

ROC800 offers RS-232 and optional RS-422/485 communication interfaces. The LOI
connector is a serial interface despite its RJ-45 form:
[ROC800 LOI wiring guide](https://www3.emersonprocess.com/remote/mobile/flipbook/a6175_ROC800-series_remote_operations_controller_instruction_manual/files/assets/basic-html/page-127.html).

A physical serial test needs an adapter matching the selected electrical interface,
the controller-specific cable/pinout, and matching line settings. Establish pinout
and baud/parity/stop bits from the exact controller/module manual. Do not substitute
a TTL UART for an RS-232 interface or assume an RJ-45 serial port is Ethernet.

The `serial` feature provides `SerialConnector` with explicit line options. It
has compiled in local Windows/Linux all-feature checks, including Rust 1.88;
the hosted CI matrix is configured but its release-commit results remain pending.
Physical serial I/O has not been validated. See [release evidence](RELEASE.md).
The current automated byte-stream seam uses Tokio duplex streams; a serial fixture
over Unix pseudo-terminals or paired physical adapters remains an additional test.

## Acceptance sequence

1. Verify core wire behavior against fixed specification-derived fixtures.
2. Verify session behavior over the injectable in-memory stream.
3. Verify the managed client over loopback TCP, including malformed/fragmented
   replies, rejected setup, outage, cancellation, queue pressure and shutdown.
4. Record a physical controller's model, firmware, addressing and security mode.
5. Run clock/parameter reads on the configured Ethernet channel and compare with
   independent controller software or packet captures.
6. Repeat on the selected serial channel before claiming serial interoperability.

Passing the simulator suite establishes our implementation's behavior; it does not
establish compatibility with firmware, undocumented quirks or physical line wiring.
