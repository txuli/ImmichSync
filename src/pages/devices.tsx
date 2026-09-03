import Database from '@tauri-apps/plugin-sql';
import { useEffect, useState } from 'react';
import ImmichForm from '../components/ImmichForm';
import Options from '../components/options';
import device from '../assets/devices.svg';
import type { DbDevice } from '../types';

function DeviceIcon({ className }: { className?: string }) {
    return <img src={device} alt="Device" className={className} />
}

export default function decices() {
    const [devices, setDevices] = useState<DbDevice[]>([]);

    async function loadData() {
        try {
            const db = await Database.load('sqlite:immichsync.db')
            const rows = await db.select<DbDevice[]>(
                'SELECT * FROM devices'
            );
            setDevices(rows);
            console.log(rows)
        } catch {

        }
    }

    useEffect(() => {
        loadData()
    }, [])

    async function handleRemove(id: number) {
        try {
            const db = await Database.load('sqlite:immichsync.db')
            await db.execute('DELETE FROM devices WHERE id = ?', [id]);
            setDevices((prev) => prev.filter((d) => d.id !== id));
        } catch {

        }
    }

    return (
        <div className="relative h-full bg-[#15171C]">
            <div className="p-6">
                <h2 className="text-2xl font-semibold">Known devices</h2>
                <p className="text-gray-400 mt-2">Modify your saved devices</p>
            </div>
            <div className="border-t-2 w-full border-t-[#272A31] absolute"></div>
            <div className="mt-4 px-4 sm:px-6 space-y-3">
                {devices.length === 0 && (
                    <ImmichForm>
                        <p className="text-sm text-gray-500 my-4">No devices saved yet.</p>
                    </ImmichForm>
                )}

                {devices.map((d) => (
                    <ImmichForm key={d.id}>
                        <div className="flex items-center gap-2 my-3">
                            <div className="w-8 h-8 rounded-md bg-[#5B8DEF]/10 flex items-center justify-center shrink-0">
                                <DeviceIcon className="w-4 h-4" />
                            </div>
                            <div className="min-w-0 flex-1">
                                <p className="text-xs text-gray-400 leading-tight">Device</p>
                                <p className="text-sm font-medium text-gray-100 truncate leading-tight">{d.device}</p>
                            </div>
                            <button
                                type="button"
                                onClick={() => handleRemove(d.id)}
                                className="text-xs text-gray-500 hover:text-red-500 transition-colors shrink-0"
                            >
                                Remove
                            </button>
                        </div>

                        <form className="grid gap-1 my-3 min-w-0">
                            <label htmlFor={`album-${d.id}`} className="text-sm">Destination album</label>
                            <input
                                id={`album-${d.id}`}
                                type="text"
                                defaultValue={d.albumName}
                                placeholder="e.g. Family USB backups"
                                className="w-full min-w-0 py-1!"
                            />
                        </form>

                        <div className="[&>div]:py-2">
                            <Options
                                checked={!!d.direct}
                                title="Automatic Sync"
                                description="Sync this drive automatically to immich."
                                onChange={() => { }}
                            />
                        </div>
                    </ImmichForm>
                ))}
            </div>
        </div>
    )
}
