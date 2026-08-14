import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "./App";

describe("Graph canvas", () => {
  it("shows Sidecar version after a successful handshake and renders an empty Graph", () => {
    render(<App handshake={{ status: "ok", version: "0.1.0" }} />);
    expect(screen.getByTestId("sidecar-status")).toHaveTextContent("Sidecar 0.1.0");
    expect(screen.getByTestId("graph-canvas")).toBeInTheDocument();
    expect(screen.queryByText(/Connecting/)).not.toBeInTheDocument();
  });

  it("shows a Sidecar error instead of a version", () => {
    render(
      <App handshake={{ status: "error", message: "Sidecar failed to start" }} />,
    );
    expect(screen.getByTestId("sidecar-status")).toHaveTextContent(
      "Sidecar failed to start",
    );
  });
});
