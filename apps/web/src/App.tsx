import { useState } from "react";

export default function App() {
  const [url, setUrl] = useState("");
  const [loading, setLoading] = useState(false);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!url.trim()) return;

    setLoading(true);

    // Trigger download by navigating to the API endpoint
    window.location.href = `http://localhost:3001/api/download?url=${encodeURIComponent(url)}`;

    // Reset after a short delay
    setTimeout(() => {
      setLoading(false);
      setUrl("");
    }, 2000);
  };

  return (
    <div style={styles.container}>
      <h1 style={styles.title}>YouTube to MP3</h1>
      <form onSubmit={handleSubmit} style={styles.form}>
        <input
          type="text"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          placeholder="Paste YouTube URL here..."
          style={styles.input}
          disabled={loading}
          autoFocus
        />
        <button
          type="submit"
          style={{
            ...styles.button,
            opacity: loading || !url.trim() ? 0.6 : 1,
            cursor: loading || !url.trim() ? "not-allowed" : "pointer",
          }}
          disabled={loading || !url.trim()}
        >
          {loading ? "Downloading..." : "Download MP3"}
        </button>
      </form>
      <p style={styles.hint}>Press Enter or click the button to download</p>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    minHeight: "100vh",
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    justifyContent: "center",
    fontFamily: "system-ui, -apple-system, sans-serif",
    backgroundColor: "#0a0a0a",
    color: "#fafafa",
    padding: "20px",
    margin: 0,
  },
  title: {
    fontSize: "2.5rem",
    marginBottom: "2rem",
    fontWeight: 600,
  },
  form: {
    display: "flex",
    flexDirection: "column",
    gap: "1rem",
    width: "100%",
    maxWidth: "500px",
  },
  input: {
    padding: "1rem",
    fontSize: "1rem",
    borderRadius: "8px",
    border: "1px solid #333",
    backgroundColor: "#1a1a1a",
    color: "#fafafa",
    outline: "none",
  },
  button: {
    padding: "1rem",
    fontSize: "1rem",
    borderRadius: "8px",
    border: "none",
    backgroundColor: "#ff0050",
    color: "#fff",
    fontWeight: 600,
    transition: "opacity 0.2s",
  },
  hint: {
    color: "#666",
    marginTop: "1rem",
    fontSize: "0.875rem",
  },
};
