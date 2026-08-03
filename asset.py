
import os
import requests
from datetime import datetime
from dotenv import load_dotenv
from metadata import get_metadata,  get_photo_dates

load_dotenv()

IMMICH_URL = os.getenv('IMMICH_URL')
IMMICH_API_KEY = os.getenv('IMMICH_API_KEY')

def createAlbum():
    url = f'{IMMICH_URL}/api/albums'
    name = str(datetime.now()) + "-album"
    myobj = {'albumName': name}
    response = requests.post(url, json = myobj,  headers = {"x-api-key": IMMICH_API_KEY})
    return response.json()['id']
def uploadAssets(route):
    url = f'{IMMICH_URL}/api/assets'
    metadata = get_metadata(route)
    created, modified = get_photo_dates(route, metadata)
    with open(route, "rb") as f:
        files = {'assetData': f}
        data = {
            'deviceAssetId': os.path.basename(route),
            'deviceId': 'python',
            'fileCreatedAt': created,
            'fileModifiedAt': modified,
        }
        response = requests.post(url, files=files, data=data, headers = {"x-api-key": IMMICH_API_KEY})

        return response.json()['id']
def assignAssetsAlbum(albumId, assetId):
      url = f'{IMMICH_URL}/api/albums/assets'
      myobj = {'albumIds': albumId, 'assetIds': assetId }
      response = requests.put(url, json = myobj,  headers = {"x-api-key": IMMICH_API_KEY})
      return  response.json()
