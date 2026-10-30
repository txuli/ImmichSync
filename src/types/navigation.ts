/** Pages the app can navigate to (see App.tsx and components/navBar.tsx). */
export type View = "dashboard" | "config" | "manualUpload" | "newDevice" | "knownDevice" | "device";

/** Payload of the "navigate-new-device" event, emitted when the OS notification's "Choose album" action is used. */
export interface NewDeviceNavigationPayload {
    diskName: string;
    mountPoint: string;
}

/** Payload of the "navigate-known-device" event, emitted when a saved device's notification is clicked. */
export interface KnownDeviceNavigationPayload extends NewDeviceNavigationPayload {
    albumName: string;
}
