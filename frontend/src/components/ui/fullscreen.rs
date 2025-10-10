use leptos::{html::*, prelude::*};

use crate::components::{accessibility::AccessibilityContext, ui::buttons::MenuButton};

#[component]
pub fn FullscreenButton() -> impl IntoView {
    let accessibility: AccessibilityContext = expect_context();

    view! {
        {accessibility
            .is_on_mobile()
            .then(|| {
                view! {
                    <MenuButton on:click=move |_| {
                        if accessibility.is_fullscreen() {
                            accessibility.exit_fullscreen();
                        } else {
                            accessibility.go_fullscreen();
                        }
                    }>
                        {move || match accessibility.is_fullscreen() {
                            false => {
                                view! {
                                    <svg
                                        xmlns="http://www.w3.org/2000/svg"
                                        class="h-[1em] aspect-square text-white"
                                        fill="currentColor"
                                        viewBox="0 0 384.97 384.97"
                                        stroke="currentColor"
                                        stroke-width="1"
                                    >
                                        <g id="Fullscreen_1_">
                                            <path d="M372.939,216.545c-6.123,0-12.03,5.269-12.03,12.03v132.333H24.061V24.061h132.333c6.388,0,12.03-5.642,12.03-12.03
                                            S162.409,0,156.394,0H24.061C10.767,0,0,10.767,0,24.061v336.848c0,13.293,10.767,24.061,24.061,24.061h336.848
                                            c13.293,0,24.061-10.767,24.061-24.061V228.395C384.97,221.731,380.085,216.545,372.939,216.545z" />
                                            <path d="M372.939,0H252.636c-6.641,0-12.03,5.39-12.03,12.03s5.39,12.03,12.03,12.03h91.382L99.635,268.432
                                            c-4.668,4.668-4.668,12.235,0,16.903c4.668,4.668,12.235,4.668,16.891,0L360.909,40.951v91.382c0,6.641,5.39,12.03,12.03,12.03
                                            s12.03-5.39,12.03-12.03V12.03l0,0C384.97,5.558,379.412,0,372.939,0z" />
                                        </g>
                                    </svg>
                                }
                                    .into_any()
                            }
                            true => {
                                view! {
                                    <svg
                                        xmlns="http://www.w3.org/2000/svg"
                                        class="h-[1em] aspect-square text-white"
                                        fill="currentColor"
                                        viewBox="0 0 1920 1920"
                                        stroke="currentColor"
                                        stroke-width="1"
                                    >
                                        <path
                                            d="M876.612 1043.388v710.171H761.27v-513.28L81.663 1920 0 1838.337l679.72-679.606H166.442v-115.343h710.171ZM1838.394 0l81.548 81.548-679.605 679.72h513.28v115.344h-710.172V166.441h115.344v513.164L1838.394 0Z"
                                            fill-rule="evenodd"
                                        />
                                    </svg>
                                }
                                    .into_any()
                            }
                        }}
                    </MenuButton>
                }
            })}
    }
}
