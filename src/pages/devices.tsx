import Database from '@tauri-apps/plugin-sql';
import { useEffect } from 'react';
export default function decices() {
    interface DbDevices {
        id: number,
        device: String,
        path: String,
        album_name: String
        direct: String
    }
   let devices: DbDevices[]
    useEffect(() => {
        async function loadData() {
            try {
                const db = await Database.load('sqlite:immichsync.db')
                 devices = await db.select(
                    'SELECT * FROM devices'
                );
                console.log(devices)


            } catch {

            }
        }
        loadData()
    }, [])


    return (
        <>

        </>
    )
}