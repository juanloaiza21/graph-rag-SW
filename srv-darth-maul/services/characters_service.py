import requests
from bs4 import BeautifulSoup
import time
import json

SWAPI_FILMS_URL = "https://swapi.info/api/films"
SWAPI_PEOPLE_URL = "https://swapi.info/api/people"
WIKI_BASE_URL = "https://starwars.fandom.com/wiki"

NAME_EXCEPTIONS = {
    "Palpatine": "Darth_Sidious",
    "Boba Fett": "Boba_Fett",
}


def get_target_character_ids():
    """
    Descarga las películas 1-6 y extrae un SET único de IDs de personajes.
    Esto asegura que solo procesemos personajes de la saga Skywalker original.
    """
    print("🕵️  Identificando personajes de Episodios 1-6...")
    try:
        response = requests.get(SWAPI_FILMS_URL)
        films = response.json()

        target_ids = set()

        for film in films:
            if 1 <= film['episode_id'] <= 6:
                for char_url in film['characters']:
                    char_id = char_url.strip('/').split('/')[-1]
                    target_ids.add(char_id)

        return target_ids
    except Exception as e:
        print(f"❌ Error obteniendo IDs de películas: {e}")
        return set()


def fetch_wiki_description(name):
    """Scraping de Wookieepedia (Inglés) para personajes"""

    if name in NAME_EXCEPTIONS:
        slug = NAME_EXCEPTIONS[name]
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
                    if len(text) > 50 and not text.lower().startswith("aside"):
                        paragraphs.append(text)
                        if len(paragraphs) >= 5:
                            break

                return "\n\n".join(paragraphs)
        return "Description not found on Wiki."
    except Exception as e:
        return f"Wiki Error: {str(e)}"


def main():
    valid_ids = get_target_character_ids()
    if not valid_ids:
        return []

    print("📡 Descargando lista maestra de personajes...")
    try:
        response = requests.get(SWAPI_PEOPLE_URL)
        all_people = response.json()
    except Exception as e:
        print(f"❌ Error conectando a SWAPI People: {e}")
        return []

    processed_characters = []

    print("⚡ Procesando y filtrando personajes...")

    for person in all_people:
        person_id = person['url'].strip('/').split('/')[-1]

        if person_id in valid_ids:
            name = person['name']
            print(f"   👤 Procesando: {name} (ID: {person_id})")

            wiki_text = fetch_wiki_description(name)

            homeworld_url = person.get('homeworld')
            homeworld_id = homeworld_url.strip(
                '/').split('/')[-1] if homeworld_url else None

            char_obj = {
                "id": f"char_{person_id}",
                "original_swapi_id": person_id,
                "name": name,
                "wiki_description": wiki_text,
                "birth_year": person.get('birth_year'),
                "gender": person.get('gender'),
                "height": person.get('height'),
                "mass": person.get('mass'),
                "homeworld_id": homeworld_id,
                "species_ids": [s.strip('/').split('/')[-1] for s in person.get('species', [])],
                "source": "swapi_plus_wookieepedia"
            }

            processed_characters.append(char_obj)

            time.sleep(0.5)

    print(f"\n✅ Finalizado. {len(processed_characters)} personajes listos.")
    return processed_characters


if __name__ == "__main__":
    res = main()
    if res:
        print(json.dumps(res[0], indent=2))
