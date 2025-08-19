use leptos::{html::*, prelude::*};
#[component]
pub fn PrivacyPage() -> impl IntoView {
    view! {
        <main class="bg-zinc-950 h-screen w-full my-0 mx-auto max-w-3xl flex flex-col">
            <div class="flex items-center justify-between p-4 border-b border-zinc-700 flex-shrink-0">
                <h2 class="text-xl font-bold text-amber-200">"Privacy Notice"</h2>
            </div>

            <div class="flex-1 overflow-y-auto p-6">
                <PrivacyContent />
            </div>

            <div class="p-4 border-t border-zinc-700 flex-shrink-0">
                <a href="" class="text-amber-400">
                    "Back to Home"
                </a>
            </div>
        </main>
    }
}

#[component]
pub fn PrivacyContent() -> impl IntoView {
    view! {
        <div class="max-w-3xl mx-auto p-6 text-gray-200">
            <h1 class="text-2xl font-bold mb-4">"Privacy Notice"</h1>
            <p>
                <strong>"Last Updated:"</strong>
                " 18 August 2025"
            </p>

            <h2 class="text-xl font-semibold mt-4 mb-2">"1. Introduction"</h2>
            <p>
                "This Privacy Notice explains how " <strong>"Grégoire Naisse"</strong>
                " (“I”, “me”, “my”) collects, uses, and protects your personal data when you use this website and services (“Service”). By using the Service, you consent to this Privacy Notice."
            </p>

            <h2 class="text-xl font-semibold mt-4 mb-2">"2. Personal Data Collected"</h2>
            <ul>
                <li>"Email address (optional) – used only for password recovery."</li>
                <li>"No other personal information is collected through the Service."</li>
            </ul>

            <h2 class="text-xl font-semibold mt-4 mb-2">"3. How Data is Stored"</h2>
            <ul>
                <li>
                    "Email addresses are stored securely in the Service's backend and are never shared with third parties."
                </li>
                <li>
                    "Other data, such as session information, is stored in your browser's local storage."
                </li>
            </ul>

            <h2 class="text-xl font-semibold mt-4 mb-2">"4. Cookies and Tracking"</h2>
            <ul>
                <li>"This Service does not use cookies for tracking or analytics."</li>
                <li>
                    "Third-party CAPTCHA (Turnstile) may temporarily process your IP for verification purposes, but no personal data is stored by me."
                </li>
            </ul>

            <h2 class="text-xl font-semibold mt-4 mb-2">"5. Third-Party Services"</h2>
            <ul>
                <li>
                    "CAPTCHA verification is performed via Turnstile. Please review Turnstile’s privacy policy for details."
                </li>
            </ul>

            <h2 class="text-xl font-semibold mt-4 mb-2">"6. Data Security"</h2>
            <p>
                "I implement reasonable measures to protect personal data from unauthorized access, loss, or misuse. However, no system is completely secure, and I cannot guarantee absolute security."
            </p>

            <h2 class="text-xl font-semibold mt-4 mb-2">"7. Your Rights"</h2>
            <ul>
                <li>
                    "You may request deletion of your email from the system at any time by contacting me via my GitHub profile: "
                    <a href="https://github.com/gregouar" class="text-amber-400 underline">
                        "https://github.com/gregouar"
                    </a> "."
                </li>
                <li>
                    "Since no other personal data is collected, there are no other records to access or correct."
                </li>
            </ul>

            <h2 class="text-xl font-semibold mt-4 mb-2">"8. Changes to Privacy Notice"</h2>
            <p>
                "I may update this Privacy Notice occasionally. The “Last Updated” date reflects the most recent version. Continued use of the Service constitutes acceptance of any changes."
            </p>

            <h2 class="text-xl font-semibold mt-4 mb-2">"9. Governing Law"</h2>
            <p>
                "This Privacy Notice is governed by Belgian law and applicable European Union regulations."
            </p>
        </div>
    }
}
