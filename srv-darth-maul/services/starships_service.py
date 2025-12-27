import requests
from bs4 import BeautifulSoup
import time
import json

SWAPI_FILMS_URL = "https://swapi.info/api/films"
SWAPI_STARSHIPS_URL = "https://swapi.info/api/starships"
WIKI_BASE_URL = "https://starwars.fandom.com/wiki"

STARSHIP_NAME_EXCEPTIONS = {
    # El nombre canónico completo
    "TIE/LN starfighter": "TIE/ln_space_superiority_starfighter",
    "X-wing": "X-wing_starfighter",
    # A menudo se refiere a este
    "Jedi starfighter": "Delta-7_Aethersprite-class_light_interceptor",
    "Slave I": "Slave_I",
    "Imperial shuttle": "Lambda-class_T-4a_shuttle"
}


def get_target_starship_ids():
    """
    Obtiene los IDs de naves que aparecen EXCLUSIVAMENTE en Episodios 1-6
    """
    print("🚀 Identificando naves de la Saga Skywalker (Ep 1-6)...")
    try:
        response = requests.get(SWAPI_FILMS_URL)
        films = response.json()

        target_ids = set()

        for film in films:
            if 1 <= film['episode_id'] <= 6:
                for ship_url in film['starships']:
                    ship_id = ship_url.strip('/').split('/')[-1]
                    target_ids.add(ship_id)

        print(f"🌌 Se encontraron {len(target_ids)} naves relevantes.")
        return target_ids
    except Exception as e:
        print(f"❌ Error obteniendo IDs de películas: {e}")
        return set()


def fetch_wiki_data(name):
    """Scraping de Wookieepedia para Naves"""

    if name in STARSHIP_NAME_EXCEPTIONS:
        slug = STARSHIP_NAME_EXCEPTIONS[name]
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
                    if len(text) > 40 and not text.lower().startswith("aside"):
                        paragraphs.append(text)
                        if len(paragraphs) >= 5:
                            break

                return "\n\n".join(paragraphs)
        return "Technical specifications available, but description not found."
    except Exception as e:
        return f"Wiki Error: {str(e)}"


def main():
    valid_ids = get_target_starship_ids()
    if not valid_ids:
        return []

    print("📡 Descargando catálogo completo de naves...")
    try:
        response = requests.get(SWAPI_STARSHIPS_URL)
        all_ships = response.json()
    except Exception as e:
        print(f"❌ Error conectando a SWAPI Starships: {e}")
        return []

    processed_ships = []

    print("🛠️  Procesando naves en el astillero...")

    for ship in all_ships:
        ship_id = ship['url'].strip('/').split('/')[-1]

        if ship_id in valid_ids:
            name = ship['name']
            print(f"   🛸 Procesando: {name} (ID: {ship_id})")

            wiki_text = fetch_wiki_data(name)

            pilot_ids = [p.strip('/').split('/')[-1]
                         for p in ship.get('pilots', [])]
            film_ids = [f.strip('/').split('/')[-1]
                        for f in ship.get('films', [])]

            ship_obj = {
                "id": f"starship_{ship_id}",
                "original_swapi_id": ship_id,
                "name": name,
                "model": ship.get('model'),
                "manufacturer": ship.get('manufacturer'),
                "wiki_description": wiki_text,
                "cost_in_credits": ship.get('cost_in_credits'),
                "length": ship.get('length'),
                "max_atmosphering_speed": ship.get('max_atmosphering_speed'),
                "crew": ship.get('crew'),
                "passengers": ship.get('passengers'),
                "cargo_capacity": ship.get('cargo_capacity'),
                "hyperdrive_rating": ship.get('hyperdrive_rating'),
                "starship_class": ship.get('starship_class'),
                "pilot_ids": pilot_ids,
                "film_ids": film_ids,
                "source": "swapi_plus_wookieepedia"
            }

            processed_ships.append(ship_obj)
            time.sleep(0.5)

    return processed_ships


if __name__ == "__main__":
    res = main()
    if res:
        print(json.dumps(res[0], indent=2))
