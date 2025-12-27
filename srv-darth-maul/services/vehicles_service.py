import requests
from bs4 import BeautifulSoup
import time
import json

SWAPI_FILMS_URL = "https://swapi.info/api/films"
SWAPI_VEHICLES_URL = "https://swapi.info/api/vehicles"
WIKI_BASE_URL = "https://starwars.fandom.com/wiki"

VEHICLE_NAME_EXCEPTIONS = {
    "Sand Crawler": "Sandcrawler",
    "T-16 skyhopper": "T-16_skyhopper",
    "X-34 landspeeder": "X-34_landspeeder",
    "Snowspeeder": "T-47_airspeeder",
    "AT-AT": "All_Terrain_Armored_Transport",
    "AT-ST": "All_Terrain_Scout_Transport",
    "Storm IV Twin-Pod Cloud Car": "Storm_IV_Twin-Pod_cloud_car",
    "Sail Barge": "Khetanna",
    "Bantha-II cargo skiff": "Bantha-II_cargo_skiff",
    "TIE/in interceptor": "TIE/in_interceptor",
    "Imperial Speeder Bike": "74-Z_speeder_bike",
    "Vulture Droid": "Variable_Geometry_Self-Propelled_Battle_Droid_Mark_I"
}


def get_target_vehicle_ids():
    """
    Obtiene los IDs de vehículos que aparecen en Episodios 1-6.
    """
    print("🚜 Identificando vehículos terrestres/atmosféricos (Ep 1-6)...")
    try:
        response = requests.get(SWAPI_FILMS_URL)
        films = response.json()

        target_ids = set()

        for film in films:
            if 1 <= film['episode_id'] <= 6:
                for vehicle_url in film['vehicles']:
                    vehicle_id = vehicle_url.strip('/').split('/')[-1]
                    target_ids.add(vehicle_id)

        return target_ids
    except Exception as e:
        print(f"❌ Error obteniendo IDs de películas: {e}")
        return set()


def fetch_wiki_data(name):
    """Scraping de Wookieepedia para Vehículos"""

    # 1. Resolver Slug
    if name in VEHICLE_NAME_EXCEPTIONS:
        slug = VEHICLE_NAME_EXCEPTIONS[name]
    else:
        slug = name.replace(" ", "_")

    url = f"{WIKI_BASE_URL}/{slug}"

    try:
        headers = {'User-Agent': 'Mozilla/5.0'}
        response = requests.get(url, headers=headers)

        if response.status_code == 200:
            soup = BeautifulSoup(response.text, 'html.parser')
            parser = soup.find('div', {'class': 'mw-parser-output'})

            if parser:
                paragraphs = []
                for p in parser.find_all('p', recursive=False):
                    text = p.get_text(strip=True)
                    # Filtro de calidad
                    if len(text) > 40 and not text.lower().startswith("aside"):
                        paragraphs.append(text)
                        if len(paragraphs) >= 5:
                            break

                return "\n\n".join(paragraphs)
        return "Technical schematics found, but tactical description missing."
    except Exception as e:
        return f"Wiki Error: {str(e)}"


def main():
    valid_ids = get_target_vehicle_ids()
    if not valid_ids:
        return []

    print("📡 Descargando inventario del parque automotor galáctico...")
    try:
        response = requests.get(SWAPI_VEHICLES_URL)
        all_vehicles = response.json()
    except Exception as e:
        print(f"❌ Error conectando a SWAPI Vehicles: {e}")
        return []

    processed_vehicles = []

    print("🔧 Revisando mecánica y pintura...")

    for vehicle in all_vehicles:
        vehicle_id = vehicle['url'].strip('/').split('/')[-1]

        if vehicle_id in valid_ids:
            name = vehicle['name']
            print(f"   🏎️  Procesando: {name} (ID: {vehicle_id})")

            wiki_text = fetch_wiki_data(name)

            pilot_ids = [p.strip('/').split('/')[-1]
                         for p in vehicle.get('pilots', [])]
            film_ids = [f.strip('/').split('/')[-1]
                        for f in vehicle.get('films', [])]

            vehicle_obj = {
                "id": f"vehicle_{vehicle_id}",
                "original_swapi_id": vehicle_id,
                "name": name,
                "model": vehicle.get('model'),
                "manufacturer": vehicle.get('manufacturer'),
                "wiki_description": wiki_text,
                "cost_in_credits": vehicle.get('cost_in_credits'),
                "length": vehicle.get('length'),
                "max_atmosphering_speed": vehicle.get('max_atmosphering_speed'),
                "crew": vehicle.get('crew'),
                "passengers": vehicle.get('passengers'),
                "cargo_capacity": vehicle.get('cargo_capacity'),
                "vehicle_class": vehicle.get('vehicle_class'),
                "pilot_ids": pilot_ids,
                "film_ids": film_ids,
                "source": "swapi_plus_wookieepedia"
            }

            processed_vehicles.append(vehicle_obj)
            time.sleep(0.5)

    print(f"\n✅ Finalizado. {len(processed_vehicles)
                             } vehículos listos para despliegue.")
    return processed_vehicles


if __name__ == "__main__":
    res = main()
    if res:
        print(json.dumps(res[0], indent=2))
