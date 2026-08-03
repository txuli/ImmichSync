import string
import time
import ctypes
import os
import win32api
import win32con


from asset  import createAlbum, uploadAssets, assignAssetsAlbum
import pillow_heif

pillow_heif.register_heif_opener()


def get_drive_status():
    """Return a set of currently available drive letters (Windows only)."""
    devices = set()
    bitmask = ctypes.windll.kernel32.GetLogicalDrives()
    for i, label in enumerate(string.ascii_uppercase):
        if bitmask & (1 << i):
            devices.add(label)
    return devices

def monitor():
    original = get_drive_status()
    
    time.sleep(3)
    current = get_drive_status()
    IMMICH_IMAGE_FORMATS = [
    ".avif",
    ".bmp",
    ".gif",
    ".heic",
    ".heif",
    ".jp2",
    ".jpeg", ".jpg", ".jpe", ".insp",
    ".jxl",
    ".mpo",
    ".png",
    ".psd",
    ".raw",
    ".rw2",
    ".svg",
    ".tif", ".tiff",
    ".webp",
]
    added = current - original

    photo_paths = []
    for drive in added:
        for root, dirs, files in os.walk(drive + ":\\"):
            for filename in files:
                ext = os.path.splitext(filename)[1].lower()
                if ext in IMMICH_IMAGE_FORMATS:
                    photo_paths.append(os.path.join(root, filename))

    if photo_paths:
        box = win32api.MessageBox(0, 'Do you want to import the photos to immich', 'New device  detected', win32con.MB_OKCANCEL)
        if box == win32con.IDOK:
            assets = [uploadAssets(path) for path in photo_paths]
            album = [createAlbum()]
            assignAssetsAlbum(album, assets)


if __name__ == '__main__':
    while True:
        monitor()
        
 
 
 
             
                   
                       
                       
                      