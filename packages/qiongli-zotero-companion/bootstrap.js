var QiongliZoteroCompanion = {
  endpoints: [
    "/qiongli/ping",
    "/qiongli/search",
    "/qiongli/upsertItems",
    "/qiongli/collections"
  ],

  startup() {
    const Zotero = getZotero();
    if (!Zotero?.Server?.Endpoints) {
      return;
    }

    registerEndpoint(Zotero, "/qiongli/ping", ["GET"], (_postData, sendResponse) => {
      sendJson(sendResponse, 200, {
        status: "ok",
        companion: "qiongli-zotero-companion",
        version: "0.1.0",
        endpoint_version: 1,
        zotero_version: Zotero.version ?? "",
        endpoints: this.endpoints
      });
    });

    registerEndpoint(Zotero, "/qiongli/search", ["POST"], (_postData, sendResponse) => {
      sendJson(sendResponse, 501, {
        status: "error",
        error_code: "not_implemented",
        message: "Local Zotero search runtime adapter is not implemented in this companion build."
      });
    });

    registerEndpoint(Zotero, "/qiongli/upsertItems", ["POST"], (_postData, sendResponse) => {
      sendJson(sendResponse, 501, {
        status: "error",
        error_code: "not_implemented",
        message: "Local Zotero upsert runtime adapter is not implemented in this companion build."
      });
    });

    registerEndpoint(Zotero, "/qiongli/collections", ["GET"], (_postData, sendResponse) => {
      sendJson(sendResponse, 501, {
        status: "error",
        error_code: "not_implemented",
        message: "Local Zotero collection runtime adapter is not implemented in this companion build."
      });
    });
  },

  shutdown() {
    const Zotero = getZotero();
    if (!Zotero?.Server?.Endpoints) {
      return;
    }
    for (const endpoint of this.endpoints) {
      delete Zotero.Server.Endpoints[endpoint];
    }
  }
};

function startup(data, reason) {
  QiongliZoteroCompanion.startup(data, reason);
}

function shutdown(data, reason) {
  QiongliZoteroCompanion.shutdown(data, reason);
}

function install() {}

function uninstall() {}

function registerEndpoint(Zotero, path, supportedMethods, handler) {
  const endpoint = Zotero.Server.Endpoints[path] = function() {};
  endpoint.prototype = {
    supportedMethods,
    init(postData, sendResponseCallback) {
      handler(postData, sendResponseCallback);
    }
  };
}

function sendJson(sendResponseCallback, status, payload) {
  sendResponseCallback(status, "application/json", JSON.stringify(payload));
}

function getZotero() {
  try {
    return Components.classes["@zotero.org/Zotero;1"]
      .getService(Components.interfaces.nsISupports)
      .wrappedJSObject;
  } catch (_error) {
    return null;
  }
}
