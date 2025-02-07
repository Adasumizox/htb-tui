# HTB Machine Lister

This Rust-based TUI (Text User Interface) application allows you to list, filter, sort, and spawn machines from Hack The Box (HTB) directly in your terminal. It utilizes the HTB API v4 and requires an HTB application API key.
To get your application token you need to visit your profile, click on "Profile Settings" tab ->"App Tokens" in right bottom corner ->"Create App Token" button

# Features

*   **List Machines:** Displays a list of both active and retired HTB machines.
*   **Filtering:**
    *   No Filter (Show All)
    *   User *Not* Owns
    *   Root *Not* Owns
    *   User and Root *Not* Owns
*   **Sorting:**
    *   Difficulty
    *   User Owns Count (Descending)
    *   Root Owns Count (Descending)
    *   Machine Name (Alphabetical)
*   **Spawning:** Spawn machines directly from the TUI (if you have the necessary HTB subscription).
*   **Interactive:** Navigate the list using arrow keys, and use keyboard shortcuts for filtering, sorting and spawning.
*   **Real-time Status:** Shows whether a machine is active or inactive.
* **User/Root Owns Indicators:** Displays ✓ or empty space whether the user owns user/root flag.

## Prerequisites

*   **Rust:**  You need the Rust programming language and Cargo (the package manager) installed.  Get it from [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).
*   **Hack The Box API Key:** You need a valid HTB *application* API key.  You can obtain one from your HTB account settings (not the invitation code). It should *not* be your personal API token.

## Libraries

*   [**Ratatui**](https://ratatui.rs/) 
*   [**Crossterm**](https://docs.rs/crossterm/latest/crossterm/)
*   [**Reqwest**](https://docs.rs/reqwest/latest/reqwest/)
*   [**Serde**](https://serde.rs/)
*   [**Serde_json**](https://github.com/serde-rs/json)
*   [**Tokio**](https://tokio.rs/)

## Installation

1.  **Clone the repository:**

    ```bash
    git clone <repository_url>  # Replace <repository_url>
    cd <repository_directory>   # Replace <repository_directory>
    ```

2.  **Set the HTB API Key:**

    *   The application reads the API key from the `HTB_API_KEY` environment variable. You can set it in a few ways:
        *   **Temporarily (for the current shell session):**

            ```bash
            export HTB_API_KEY="your_htb_api_key"
            ```
        *  **Permanently (recommended):** Add the `export` line to your shell's configuration file (e.g., `~/.bashrc`, `~/.zshrc`, or `~/.profile`).  You'll need to restart your terminal or source the file (e.g., `source ~/.bashrc`) for the changes to take effect.

            ```bash
            echo 'export HTB_API_KEY="your_htb_api_key"' >> ~/.bashrc
            source ~/.bashrc
            ```

        * **Using .env file (optional but convenient):** Create a file named `.env` in the project's root directory and add the following line:
           ```
           HTB_API_KEY=your_htb_api_key
           ```
           If you use this method, you might want to add `.env` to your `.gitignore` file to avoid accidentally committing your API key. *You don't need to install any additional crates; the `env::var` function in Rust will pick up the `.env` file.*

3.  **Build and Run:**

    ```bash
    cargo run
    ```

## Usage

*   **Navigation:**
    *   **Up/Down Arrows:** Move the selection in the list.
*   **Filtering:**
    *   **`f` key:** Cycle through the filter options (None, User Owns, Root Owns, User & Root Owns, User Not Owns, Root Not Owns, User & Root Not Owns).
*   **Sorting:**
    *   **`s` key:** Cycle through the sort options (Difficulty, User Owns, Root Owns, Name).
*   **Spawning:**
    *   **`Enter` key:** Spawn the currently selected machine.  A message will indicate success or failure.
*   **Quitting:**
    *   **`q` key:** Exit the application.

The application displays the current filter and sort criteria in the title bar of the machine list.  It also shows an "Active" or "Inactive" status for each machine, as well as "U" and "R" indicators for user and root owns.

## Troubleshooting

*   **`HTB_API_KEY` not found:** Make sure you've set the `HTB_API_KEY` environment variable correctly. Double-check for typos.
*   **API Errors:** If you see errors related to the API (e.g., "401 Unauthorized"), verify that your API key is valid and has the necessary permissions.
*   **Terminal Issues:** If you experience display issues, ensure your terminal emulator supports ANSI escape codes. Most modern terminals do.

## Potential Enhancements

*   **Configuration File:** Load settings (like API key, default filter, default sort) from a configuration file.
*   **Machine Details:** Show more detailed information about a machine in a separate pane or popup (e.g., IP address after spawning, description, difficulty breakdown).
*   **Targeted Updates:** Instead of refetching the entire machine list after spawning, update only the relevant machine's data. This would greatly improve performance.
*   **Search Functionality:** Allow users to search for machines by name or other criteria.
*   **Asynchronous Updates:** Use asynchronous tasks (tokio) more effectively to avoid blocking the UI while fetching data.

## Contributing

Pull requests are welcome! Please follow good Rust coding practices and include tests if possible.

## License

This project is licensed under the MIT License.
