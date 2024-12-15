from difflib import HtmlDiff

def compare_html_files(file1, file2, output_file="comparison.html"):
    """
    Compare two HTML files and save the differences as an HTML report.

    Args:
        file1 (str): Path to the first HTML file.
        file2 (str): Path to the second HTML file.
        output_file (str): Path to save the HTML difference report.
    """
    try:
        # Read the contents of the files
        with open(file1, "r", encoding="utf-8") as f1, open(file2, "r", encoding="utf-8") as f2:
            content1 = f1.readlines()
            content2 = f2.readlines()

        # Generate HTML differences
        diff = HtmlDiff().make_file(content1, content2, file1, file2)

        # Save the difference report to an HTML file
        with open(output_file, "w", encoding="utf-8") as f:
            f.write(diff)

        print(f"Differences saved to {output_file}")
    except Exception as e:
        print(f"Error comparing files: {e}")


# Example usage
file1 = "www.bbc.com_news_first.html"
file2 = "www.bbc.com_news_second.html"
output_file = "www.bbc_comparison.html"

compare_html_files(file1, file2, output_file)
