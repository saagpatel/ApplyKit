import { describe, expect, it } from "vitest";

import "../styles/global.css";

describe("Vite environment declarations", () => {
  it("allows stylesheet side-effect imports in the TypeScript test graph", () => {
    expect(true).toBe(true);
  });
});
