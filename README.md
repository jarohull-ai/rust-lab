# 🚀 Rust Lab – The Arduino of Rust

**Visual Graph ↔ Real Rust Code ↔ Execution**  
Hybrydowe środowisko do nauki, prototypowania i generowania produkcyjnego kodu w języku Rust.

> *"Move from Action → Modification → Implementation. Instead of fighting the compiler for hours, build working systems with atomic blocks, then dive into the code to understand why it works."*

---

## 📌 Project Status

- ✅ **Core Engine & Nodes** – 11 węzłów atomowych (I/O, konwersja, parsowanie, sterowanie).
- ✅ **Execution Planner (DAG)** – Topologiczny resolver zależności danych oraz sterowania (`after`).
- ✅ **Control Flow & Branching** – Węzeł `condition` z dynamicznym trasowaniem `__next__`/`__skip__` oraz inteligentnym przycinaniem nieaktywnych gałęzi (*branch pruning*).
- ✅ **Typed Pins & Isolation** – Ścisła kontrola typów pinów (`String`, `FilePath`, `Json`, `Bool`, `Number`, `Any`) wraz z węzłami konwertującymi.
- ✅ **Construction Mode (Codegen)** – Generator kompilowalnego, samodzielnego projektu Cargo (`Cargo.toml` + `src/main.rs` + `src/nodes.rs`) gotowego do `cargo run`.
- ✅ **Visual Canvas (React Flow + CodeMirror)** – Interaktywny webowy edytor grafu, podgląd kodu Rusta na żywo w czasie rzeczywistym i konsola diagnostyczna.
- ✅ **Copilot AI (SEED Rule Engine)** – Deterministyczny silnik rekomendacji i walidacji kontekstowej (0 kosztów LLM) zintegrowany z paskiem narzędzi UI.
- ✅ **100% Green Test Suite** – 61/61 testów jednostkowych i integracyjnych, 0 ostrzeżeń `cargo clippy`.

---

## 🧱 Architecture

```
rust-lab/
├── crates/
│   ├── lab-node-sdk/   # Core SDK: Trait ViperNode, NodeSpec, TypedPin, PinType
│   ├── runtime/        # DAG Planner, Validator, Executor, Codegen, Copilot SEED Engine
│   └── api/            # Axum HTTP Server (/api/*), testy integracyjne, static UI hosting
├── ui/                 # React 19, @xyflow/react, CodeMirror (Rust syntax), Tailwind CSS v4
├── examples/           # Gotowe do uruchomienia pliki grafów (JSON)
└── data/               # Przestrzeń robocza na pliki wejściowe i wyjściowe
```

---

## 🧩 11 Atomic Nodes

| Node | Kategoria | Wejścia (Inputs) | Wyjścia (Outputs) | Opis |
|---|---|---|---|---|
| `http_get` | I/O & Network | `url: String` *(req)* | `body: String`, `status: Number` | Asynchroniczne pobieranie zasobów po HTTP |
| `fs_read` | I/O | `path: FilePath` *(req)* | `content: String` | Bezpieczny odczyt pliku tekstowego z dysku |
| `fs_write` | I/O | `path: FilePath`, `content: String` | `bytes_written: Number` | Zapis przetworzonej treści do pliku |
| `log_output` | Output | `text: Any` *(req)* | — | Wypisanie tekstu/obiektu na standardowe wyjście |
| `json_parse` | Data | `json_str: Json/String`, `path: String` | `value: Json` | Parsowanie JSON i ekstrakcja wartości po kluczu kropkowym |
| `text_clean` | Text | `text: String` *(req)* | `cleaned: String` | Usuwanie zbędnych spacji, kapitalizacja, trim |
| `condition` | Control | `value: Bool` *(req)* | `result: Bool` | Warunkowe rozgałęzienie (`true_branch`, `false_branch`) |
| `delay` | Control | `duration: String` *(req)* | — | Asynchroniczne uśpienie potoku (np. `500ms`, `2s`) |
| `string_to_path` | Converter | `path: String` *(req)* | `path: FilePath` | Jawna walidacja i konwersja `String` na `FilePath` |
| `json_to_string` | Converter | `value: Json` *(req)* | `text: String` | Bezpieczna ekstrakcja tekstu z wartości `Json` |
| `json_to_bool` | Converter | `value: Json` *(req)* | `value: Bool` | Parsowanie flagi boolean z `Json` |

---

## 💡 Copilot AI (SEED Rule Engine)

Rust Lab wyposażony jest w **deterministyczny silnik reguł SEED**, który analizuje graf studenta w czasie rzeczywistym bez wysyłania danych do chmurowych LLM (0 kosztów, czas reakcji <1ms).

Ikona żarówki (💡 **Copilot**) na pasku narzędzi dynamicznie sygnalizuje:
1. **Reguła 1: Brakujące wejście** – Wykrywa brak wymaganych parametrów (`required`) i podpowiada wartość domyślną.
2. **Reguła 2: Osierocony węzeł** – Ostrzega o węzłach pozostawionych na canvasie bez połączeń.
3. **Reguła 3: Detekcja cyklu** – Blokuje pętle (DAG enforcement) zanim wywołają błąd w runtime.
4. **Reguła 4: Type Mismatch** – Wykrywa niezgodność typów na krawędziach i wskazuje konkretny konwerter (np. `string_to_path`).
5. **Reguła 5: Połączenie z węzła bez wyjścia** – Ostrzega przed próbą pobrania danych z węzłów czysto konsumenckich (np. `log_output`).
6. **Reguła 6: Smart Next Step** – Rekomenduje logiczne kolejne węzły dla wiszących wyjść (np. `http_get.body` → `json_parse` lub `log_output`).

Kliknięcie podpowiedzi w dymku automatycznie zaznacza i podświetla powiązany węzeł na canvasie.

---

## 📂 Gotowe Przykłady Grafów (`examples/`)

W katalogu `examples/` znajdują się przykładowe grafy gotowe do uruchomienia w CLI oraz wgrania do UI:

- **`examples/web_scraper.json`**  
  Potok Web Scraper: `http_get` (pobiera JSON z `httpbin.org`) → `json_parse` (wyciąga `slideshow.author`) → `json_to_string` → `text_clean` (uppercase) → `log_output`.
- **`examples/file_pipeline.json`**  
  Potok plikowy: `fs_read` (odczytuje `data/sample.txt`) → `text_clean` → `string_to_path` → `fs_write` (zapisuje oczyszczony tekst do `data/output.txt`) → `log_output`.
- **`examples/http_pipeline.json`**  
  Potok ekstrakcji danych z API z parsowaniem tytułu prezentacji.
- **`examples/condition_delay.json`**  
  Rozgałęzienie logiczne: sprawdzenie uprawnień `is_admin`, wykonanie asynchronicznego opóźnienia (`delay`) i przycięcie gałęzi alternatywnej.

---

## ⚡ Szybki Start

### 1. Uruchomienie Visual Canvas (Web UI + API)

```bash
# Uruchomienie serwera API (serwuje endpointy oraz zbudowany interfejs webowy):
cargo run -p api

# Otwórz przeglądarkę pod adresem:
# 👉 http://localhost:3030
```

Dla pracy deweloperskiej z hot-reloadem frontendu (Vite):
```bash
# Terminal 1: Backend API
cargo run -p api

# Terminal 2: Vite Dev Server
cd ui && npm run dev
# 👉 http://localhost:5173
```

### 2. Uruchomienie grafu przez CLI

```bash
# Uruchomienie potoku plikowego
cargo run -p runtime -- examples/file_pipeline.json

# Uruchomienie potoku web scrapera
cargo run -p runtime -- examples/web_scraper.json
```

### 3. Eksport do samodzielnego projektu Cargo (Construction Mode)

```bash
# Wygenerowanie kompletnego projektu w katalogu ./my-app
cargo run -p runtime -- --export ./my-app examples/web_scraper.json

# Uruchomienie wygenerowanego kodu w 100% natywnym Rust/Cargo
cd my-app
cargo run
```

---

## 🧪 Testy i Jakość Kodu

Wszystkie crate'y są w pełni przetestowane pod kątem zgodności specyfikacji, planowania DAG, walidacji typów i reguł Copilota:

```bash
# Uruchomienie wszystkich 61 testów w workspace
cargo test

# Weryfikacja lintera Clippy (0 ostrzeżeń)
cargo clippy --all-targets --all-features -- -D warnings
```

---

## 📄 Licencja

Projekt udostępniany jest na warunkach licencji:
- MIT License ([LICENSE-MIT](LICENSE) lub http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) lub http://www.apache.org/licenses/LICENSE-2.0)
