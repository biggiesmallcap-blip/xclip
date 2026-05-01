// Shared callable so App's keydown handler can fire the export from ExportPanel.
export const exportRef: { trigger: (() => Promise<void>) | null } = { trigger: null };
