import os
import requests

def scrape_and_save_pages(websites, output_dir="raw_scrapes"):
    """
    Fetch raw HTML from websites and save to files.
    """
    # Create output directory if it doesn't exist
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    for url in websites:
        try:
            # Fetch the page
            response = requests.get(url, timeout=10)
            response.raise_for_status()  # Raise HTTPError for bad responses

            # Save the content to a file
            domain = url.split("//")[1].split("/")[0]  # Extract domain name
            filename = os.path.join(output_dir, f"{domain}.html")
            with open(filename, "w", encoding="utf-8") as f:
                f.write(response.text)

            print(f"Saved raw HTML for {url} to {filename}")
        except requests.exceptions.RequestException as e:
            print(f"Error scraping {url}: {e}")

# Example list of websites to scrape
websites = [
    "https://www.wikipedia.org",
    "https://www.bbc.com/news",
    "https://www.si.edu",
    "https://www.data.gov",
    "https://en.wikipedia.org/wiki/Al_Capone"
]

scrape_and_save_pages(websites)
