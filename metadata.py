
import os
import datetime
from PIL import Image
from PIL.ExifTags import TAGS, GPSTAGS, IFD
from datetime import datetime, timezone
def get_metadata(file_path):
    """Return all EXIF metadata of a photo (including GPS and sub-IFDs) as an array of {tag, value} entries."""
    metadata = []
    with Image.open(file_path) as img:
        exif = img.getexif()
        for tag_id, value in exif.items():
            tag = TAGS.get(tag_id, tag_id)
            metadata.append({'tag': tag, 'value': value})

        for ifd_id in (IFD.Exif, IFD.GPSInfo, IFD.Interop):
            try:
                sub_ifd = exif.get_ifd(ifd_id)
            except KeyError:
                continue
            tag_names = GPSTAGS if ifd_id == IFD.GPSInfo else TAGS
            for tag_id, value in sub_ifd.items():
                tag = tag_names.get(tag_id, tag_id)
                metadata.append({'tag': tag, 'value': value})
    return metadata
def get_photo_dates(route, metadata):
    """Return (fileCreatedAt, fileModifiedAt) ISO 8601 strings taken from the photo's own EXIF data."""
    def find_tag(name):
        for entry in metadata:
            if entry['tag'] == name:
                return entry['value']
        return None

    def exif_to_iso(exif_dt):
        return datetime.strptime(exif_dt, '%Y:%m:%d %H:%M:%S').replace(tzinfo=timezone.utc).isoformat()

    created_raw = find_tag('DateTimeOriginal') or find_tag('DateTime')
    modified_raw = find_tag('DateTime') or created_raw

    created = exif_to_iso(created_raw) if created_raw else datetime.fromtimestamp(os.path.getctime(route), tz=timezone.utc).isoformat()
    modified = exif_to_iso(modified_raw) if modified_raw else datetime.fromtimestamp(os.path.getmtime(route), tz=timezone.utc).isoformat()
    return created, modified

