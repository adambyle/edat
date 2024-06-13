# Release checklist

Before releasing, remember to:

- Replace some File operations with fs::read_to_string
- Ignore line=0 for section redirects

Todo:
- Make sure data functions alter parent in a persistent way (avoid self.index. raw references).
