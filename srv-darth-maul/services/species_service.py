import requests
from bs4 import BeautifulSoup
import time
import json

# --- CONFIGURACIÓN ---
SWAPI_FILMS_URL = "https://swapi.info/api/films"
SWAPI_SPECIES_URL = "https://swapi.info/api/species"
WIKI_BASE_URL = "https://starwars.fandom.com/wiki"

SPECIES_NAME_EXCEPTIONS = {
    "Yoda's species": "Yoda's_species",
    "Mon Calamari": "Mon_Calamari",
    "Wookie": "Wookiee",
    "Pau'an": "Pau'an",
    "Kaminoan": "Kaminoan",
    "Droid": "Droid"
}


def get_target_species_ids():
    """
    Obtiene los IDs de especies que aparecen en Episodios 1-6.
    """
    print("🧬 Secuenciando ADN alienígena (Ep 1-6)...")
    try:
        response = requests.get(SWAPI_FILMS_URL)
        films = response.json()

        target_ids = set()

        for film in films:
            if 1 <= film['episode_id'] <= 6:
                for species_url in film['species']:
                    specie_id = species_url.strip('/').split('/')[-1]
                    target_ids.add(specie_id)

        return target_ids
    except Exception as e:
        print(f"❌ Error obteniendo IDs de películas: {e}")
        return set()


def fetch_wiki_data(name):
    """Scraping de Wookieepedia para Especies"""

    if name in SPECIES_NAME_EXCEPTIONS:
        slug = SPECIES_NAME_EXCEPTIONS[name]
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
                        if len(paragraphs) >= 10:
                            break

                return "\n\n".join(paragraphs)
        return "Biological classification exists, but cultural description is missing."
    except Exception as e:
        return f"Wiki Error: {str(e)}"


def main():
    valid_ids = get_target_species_ids()
    if not valid_ids:
        return []

    print("📡 Descargando censo biológico galáctico...")
    try:
        response = requests.get(SWAPI_SPECIES_URL)
        all_species = response.json()
    except Exception as e:
        print(f"❌ Error conectando a SWAPI Species: {e}")
        return []

    processed_species = []

    print("🧪 Analizando muestras...")

    for specie in all_species:
        specie_id = specie['url'].strip('/').split('/')[-1]

        if specie_id in valid_ids:
            name = specie['name']
            print(f"   👽 Procesando: {name} (ID: {specie_id})")

            wiki_text = fetch_wiki_data(name)

            homeworld_url = specie.get('homeworld')
            homeworld_id = homeworld_url.strip(
                '/').split('/')[-1] if homeworld_url else None

            people_ids = [p.strip('/').split('/')[-1]
                          for p in specie.get('people', [])]
            film_ids = [f.strip('/').split('/')[-1]
                        for f in specie.get('films', [])]

            specie_obj = {
                "id": f"species_{specie_id}",
                "original_swapi_id": specie_id,
                "name": name,
                "classification": specie.get('classification'),
                "designation": specie.get('designation'),
                "average_height": specie.get('average_height'),
                "average_lifespan": specie.get('average_lifespan'),
                "language": specie.get('language'),
                "skin_colors": specie.get('skin_colors'),
                "wiki_description": wiki_text,
                "homeworld_id": homeworld_id,
                "people_ids": people_ids,
                "film_ids": film_ids,
                "source": "swapi_plus_wookieepedia"
            }

            processed_species.append(specie_obj)
            time.sleep(0.5)

    print(f"\n✅ Finalizado. {len(processed_species)} especies catalogadas.")
    return processed_species


if __name__ == "__main__":
    res = main()
    if res:
        print(json.dumps(res[0], indent=2))
