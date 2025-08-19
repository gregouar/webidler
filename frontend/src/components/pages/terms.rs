use leptos::{html::*, prelude::*};

#[component]
pub fn TermsPage() -> impl IntoView {
    view! {
        <main class="bg-zinc-950 h-screen w-full my-0 mx-auto max-w-3xl flex flex-col">
            <div class="flex items-center justify-between p-4 border-b border-zinc-700 flex-shrink-0">
                <h2 class="text-xl font-bold text-amber-200">"Terms and Conditions"</h2>
            </div>

            <div class="flex-1 overflow-y-auto p-6">
                <TermsContent />
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
pub fn TermsContent() -> impl IntoView {
    view! {
        <div class="max-w-3xl mx-auto p-6 text-gray-200">
            <h1 class="text-2xl font-bold mb-4">"Terms and Conditions"</h1>
            <p>
                <strong>"Last Updated:"</strong>
                " 18 August 2025"
            </p>

            <h2 class="text-xl font-semibold mt-4 mb-2">"1. Introduction"</h2>
            <p>
                "These Terms and Conditions (“Terms”) govern your use of this website and services (“Service”) provided by "
                <strong>"Grégoire Naisse"</strong>
                " (“I”, “me”, “my”). By using this Service, you agree to these Terms. If you do not agree, please do not use the Service."
            </p>

            <h2 class="text-xl font-semibold mt-4 mb-2">"2. Contact Information"</h2>
            <p>
                "As an individual, I do not share my personal address. You may contact me via my GitHub profile: "
                <a href="https://github.com/gregouar" class="text-amber-400 underline">
                    "https://github.com/gregouar"
                </a> "."
            </p>

            <h2 class="text-xl font-semibold mt-4 mb-2">"3. Use of the Service"</h2>
            <ul>
                <li>"The Service is provided for personal and entertainment purposes only."</li>
                <li>
                    "You may use the Service only in a lawful manner and in accordance with these Terms."
                </li>
                <li>"You must not upload or share unlawful, offensive, or harmful content.."</li>
                <li>
                    "You must not use the Service to harm, disrupt, or attempt unauthorized access to the Service or related systems."
                </li>
            </ul>

            <h2 class="text-xl font-semibold mt-4 mb-2">"4. Account Information"</h2>
            <ul>
                <li>"Registration is needed to access the Service."</li>
                <li>"If you provide an email address, it is used only for password recovery."</li>
                <li>
                    "You are responsible for maintaining the confidentiality of your account credentials."
                </li>
            </ul>

            <h2 class="text-xl font-semibold mt-4 mb-2">"5. Intellectual Property"</h2>
            <ul>
                <li>
                    "All content, design, and code of the Service are © Grégoire Naisse, unless otherwise noted."
                </li>
                <li>
                    "You may not copy, distribute, or create derivative works without permission."
                </li>
                <li>"Note: Some content has been generated with AI assistance."</li>
            </ul>

            <h2 class="text-xl font-semibold mt-4 mb-2">"6. Privacy and Data Protection"</h2>
            <ul>
                <li>
                    "Personal data is processed in accordance with our "
                    <a href="privacy" class="text-amber-400 underline">
                        "Privacy Notice"
                    </a>"."
                </li>
                <li>"No personal data is required to use the service."</li>
                <li>"An email address may optionally be provided for account recovery."</li>
            </ul>

            <h2 class="text-xl font-semibold mt-4 mb-2">"7. Limitation of Liability"</h2>
            <ul>
                <li>"The Service is provided “as is” without warranties of any kind."</li>
                <li>"I am not liable for any damages resulting from your use of the Service."</li>
                <li>"I do not guarantee uninterrupted availability."</li>
                <li>"Use the Service at your own risk."</li>
            </ul>

            <h2 class="text-xl font-semibold mt-4 mb-2">"8. Termination"</h2>
            <p>
                "I may suspend or terminate your access to the Service at any time for any reason, including violations of these Terms."
            </p>

            <h2 class="text-xl font-semibold mt-4 mb-2">"9. Changes to Terms"</h2>
            <p>
                "
                I reserve the right to update these Terms.
                Users will be informed of significant changes in a timely manner.
                Continued use of the service after changes implies acceptance.
                "
            </p>

            <h2 class="text-xl font-semibold mt-4 mb-2">"10. Governing Law"</h2>
            <p>
                "These Terms are governed by Belgian law and applicable European Union regulations."
            </p>
        </div>
    }
}
