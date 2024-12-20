let liveSocket = new LiveSocket("/live", Socket, {
  hooks: { ...Hooks, CopyMixInstallationHook, OXCExampleObjectHook },
  longPollFallbackMs: 2500,
  params: { _csrf_token: csrfToken },
});
