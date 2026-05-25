export async function extract_pdf_book(file) {
    const arrayBuffer = await file.arrayBuffer();

    const pdf = await window.pdfjsLib.getDocument({ data: arrayBuffer }).promise;

    const meta = await pdf.getMetadata().catch(() => null);

    const pages = [];

    for (let i = 1; i <= pdf.numPages; i++) {
        const page = await pdf.getPage(i);
        const content = await page.getTextContent();

        const text = content.items
            .map((item) => item.str)
            .join(" ")
            .replace(/\s+/g, " ")
            .trim();

        pages.push({
            page_number: i,
            text,
            char_count: text.length
        });
    }

    return {
        metadata: meta?.info || {},
        total_pages: pdf.numPages,
        pages
    };
}