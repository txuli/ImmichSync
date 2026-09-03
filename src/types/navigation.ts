/** Pages the app can navigate to (see App.tsx and components/navBar.tsx). */
export type View = "dashboard" | "config" | "manualUpload" | "newDevice" | "device";

/** Payload of the "navigate-new-device" event, emitted when the OS notification's "Choose album" action is used. */
export interface NewDeviceNavigationPayload {
    diskName: string;
    mountPoint: string;
}
