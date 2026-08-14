import { registerActionTypes } from '@tauri-apps/plugin-notification';

await registerActionTypes([
  {
    id: 'messages',
    actions: [
      {
        id: 'reply',
        title: 'Responder',
        input: true,
        inputButtonTitle: 'Enviar',
        inputPlaceholder: 'Escribe tu respuesta...',
      },
      {
        id: 'mark-read',
        title: 'Marcar como leído',
        foreground: false,
      },
    ],
  },
]);