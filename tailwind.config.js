module.exports = {
    darkMode: "class",
    content: [
        "./src/**/*.rs",
        "./index.html",
    ],
    theme: {
        extend: {
            cursor: {
                pinch: "url('/assets/download.png') 16 16, auto",
            },
            fontFamily: {
                mono: ['"IBM Plex Mono"', 'ui-monospace', 'monospace'],
                sans: ['"IBM Plex Sans"', 'ui-sans-serif', 'system-ui', 'sans-serif'],
            },
            colors: {
                aux: "#151821",
                button: "rgb(236 72 153)",
                secbutton: ""
            }
        },
    },
    plugins: [],
};