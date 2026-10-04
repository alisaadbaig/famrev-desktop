Session persistence: WebView2 stores cookies + localStorage under the app's data dir
(keyed by bundle identifier 'ai.famrev.desktop'). The server's session cookie
(llmchat_session) therefore survives app restarts — users stay signed in until the
session expires (30 days with 'Keep me signed in', or until app close without it).
