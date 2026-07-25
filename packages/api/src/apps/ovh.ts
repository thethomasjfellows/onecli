import type { AppDefinition, OAuthExchangeResult } from "./types";

const OVH_US_API_HOST = "api.us.ovhcloud.com";

export const exchangeOvhCredentials = async (
  fields: Record<string, string>,
): Promise<OAuthExchangeResult> => {
  const { applicationKey, applicationSecret, consumerKey } = fields;

  if (!applicationKey || !applicationSecret || !consumerKey) {
    throw new Error(
      "Application Key, Application Secret, and Consumer Key are required",
    );
  }

  const maskedKey = `${applicationKey.slice(0, 4)}...${applicationKey.slice(-4)}`;

  return {
    credentials: {
      type: "ovh_api",
      applicationKey,
      applicationSecret,
      consumerKey,
    },
    scopes: ["OVHcloud API credentials"],
    metadata: {
      name: `OVHcloud US (${maskedKey})`,
      apiHost: OVH_US_API_HOST,
    },
  };
};

export const ovh: AppDefinition = {
  id: "ovh",
  name: "OVHcloud US",
  icon: "/icons/ovh.svg",
  description:
    "Manage OVHcloud U.S. infrastructure through signed API requests.",
  connectionMethod: {
    type: "credentials_import",
    fields: [
      {
        name: "applicationKey",
        label: "Application Key",
        placeholder: "OVHcloud application key",
      },
      {
        name: "applicationSecret",
        label: "Application Secret",
        placeholder: "OVHcloud application secret",
        secret: true,
      },
      {
        name: "consumerKey",
        label: "Consumer Key",
        placeholder: "OVHcloud consumer key",
        secret: true,
      },
    ],
    exchangeCredentials: exchangeOvhCredentials,
  },
  labelHint: 'e.g. "qFido production"',
  available: true,
};
