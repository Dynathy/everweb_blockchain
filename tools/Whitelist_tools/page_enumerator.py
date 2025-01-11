from googlesearch import search

def get_subpages(domain, query="", num_results=20):
    """
    Use Google Search to retrieve subpages of a domain.

    Args:
        domain (str): The domain to search (e.g., "wikipedia.org").
        query (str): Additional search keywords (optional).
        num_results (int): Number of results to retrieve.

    Returns:
        list: A list of subpage URLs.
    """
    search_query = f"site:{domain} {query}"
    subpages = []

    try:
        # Perform the search
        for result in search(search_query, num_results=num_results):
            subpages.append(result)
    except Exception as e:
        print(f"Error during search: {e}")

    return subpages


def save_subpages_to_file(subpages, output_file="subpages.txt"):
    """
    Save a list of subpage URLs to a text file.

    Args:
        subpages (list): List of subpage URLs.
        output_file (str): File name to save the results.
    """
    try:
        with open(output_file, "w") as f:
            for url in subpages:
                f.write(url + "\n")
        print(f"Subpages saved to {output_file}")
    except Exception as e:
        print(f"Error saving subpages to file: {e}")


# Example Usage
if __name__ == "__main__":
    domain = "wikipedia.org"
    query = ""  # Optional: Add keywords to narrow the search
    num_results = 50  # Number of results to fetch

    # Get the subpages
    subpages = get_subpages(domain, query=query, num_results=num_results)

    if subpages:
        print(f"Found {len(subpages)} subpages:")
        for url in subpages:
            print(url)

        # Save the results to a file
        save_subpages_to_file(subpages, "wikipedia_subpages.txt")
    else:
        print("No subpages found.")
