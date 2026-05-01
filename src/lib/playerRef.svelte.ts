// Shared mutable reference to the <video> element so App and TrimControls
// can read currentTime / call play/pause without prop-drilling.
export const playerRef: { video: HTMLVideoElement | null } = { video: null };
