import { describe, expect, it } from "vitest";

import { exchangeOvhCredentials } from "./ovh";

describe("exchangeOvhCredentials", () => {
  const fields = {
    applicationKey: "app-key-1234",
    applicationSecret: "app-secret-4567",
    consumerKey: "consumer-key-8901",
  };

  it("stores the three OVHcloud credentials as one signed connection", async () => {
    await expect(exchangeOvhCredentials(fields)).resolves.toEqual({
      credentials: {
        type: "ovh_api",
        ...fields,
      },
      scopes: ["OVHcloud API credentials"],
      metadata: {
        name: "OVHcloud US (app-...1234)",
        apiHost: "api.us.ovhcloud.com",
      },
    });
  });

  it.each([["applicationKey"], ["applicationSecret"], ["consumerKey"]])(
    "rejects a missing %s",
    async (field) => {
      const incomplete = { ...fields, [field]: "" };

      await expect(exchangeOvhCredentials(incomplete)).rejects.toThrow(
        "Application Key, Application Secret, and Consumer Key are required",
      );
    },
  );
});
