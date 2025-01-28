// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="index.html"><strong aria-hidden="true">1.</strong> Introduction</a></li><li class="chapter-item expanded "><a href="config/index.html"><strong aria-hidden="true">2.</strong> Configuration</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="config/settings.html"><strong aria-hidden="true">2.1.</strong> Settings</a></li><li class="chapter-item expanded "><a href="config/partial.html"><strong aria-hidden="true">2.2.</strong> Partials</a></li><li class="chapter-item expanded "><a href="config/nested.html"><strong aria-hidden="true">2.3.</strong> Nesting</a></li><li class="chapter-item expanded "><a href="config/context.html"><strong aria-hidden="true">2.4.</strong> Context</a></li><li class="chapter-item expanded "><a href="config/struct/index.html"><strong aria-hidden="true">2.5.</strong> Structs &amp; enums</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="config/struct/default.html"><strong aria-hidden="true">2.5.1.</strong> Default values</a></li><li class="chapter-item expanded "><a href="config/struct/env.html"><strong aria-hidden="true">2.5.2.</strong> Environment variables</a></li><li class="chapter-item expanded "><a href="config/struct/extend.html"><strong aria-hidden="true">2.5.3.</strong> Extendable sources</a></li><li class="chapter-item expanded "><a href="config/struct/merge.html"><strong aria-hidden="true">2.5.4.</strong> Merge strategies</a></li><li class="chapter-item expanded "><a href="config/struct/validate.html"><strong aria-hidden="true">2.5.5.</strong> Validation rules</a></li></ol></li><li class="chapter-item expanded "><a href="config/enum/index.html"><strong aria-hidden="true">2.6.</strong> Unit-only enums</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="config/enum/default.html"><strong aria-hidden="true">2.6.1.</strong> Default variant</a></li><li class="chapter-item expanded "><a href="config/enum/fallback.html"><strong aria-hidden="true">2.6.2.</strong> Fallback variant</a></li></ol></li><li class="chapter-item expanded "><a href="config/experimental.html"><strong aria-hidden="true">2.7.</strong> Experimental</a></li></ol></li><li class="chapter-item expanded "><a href="schema/index.html"><strong aria-hidden="true">3.</strong> Schemas</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="schema/types.html"><strong aria-hidden="true">3.1.</strong> Types</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="schema/array.html"><strong aria-hidden="true">3.1.1.</strong> Arrays</a></li><li class="chapter-item expanded "><a href="schema/boolean.html"><strong aria-hidden="true">3.1.2.</strong> Booleans</a></li><li class="chapter-item expanded "><a href="schema/enum.html"><strong aria-hidden="true">3.1.3.</strong> Enums</a></li><li class="chapter-item expanded "><a href="schema/float.html"><strong aria-hidden="true">3.1.4.</strong> Floats</a></li><li class="chapter-item expanded "><a href="schema/integer.html"><strong aria-hidden="true">3.1.5.</strong> Integers</a></li><li class="chapter-item expanded "><a href="schema/literal.html"><strong aria-hidden="true">3.1.6.</strong> Literals</a></li><li class="chapter-item expanded "><a href="schema/null.html"><strong aria-hidden="true">3.1.7.</strong> Nulls</a></li><li class="chapter-item expanded "><a href="schema/object.html"><strong aria-hidden="true">3.1.8.</strong> Objects</a></li><li class="chapter-item expanded "><a href="schema/string.html"><strong aria-hidden="true">3.1.9.</strong> Strings</a></li><li class="chapter-item expanded "><a href="schema/struct.html"><strong aria-hidden="true">3.1.10.</strong> Structs</a></li><li class="chapter-item expanded "><a href="schema/tuple.html"><strong aria-hidden="true">3.1.11.</strong> Tuples</a></li><li class="chapter-item expanded "><a href="schema/union.html"><strong aria-hidden="true">3.1.12.</strong> Unions</a></li><li class="chapter-item expanded "><a href="schema/unknown.html"><strong aria-hidden="true">3.1.13.</strong> Unknown</a></li></ol></li><li class="chapter-item expanded "><a href="schema/external.html"><strong aria-hidden="true">3.2.</strong> External types</a></li><li class="chapter-item expanded "><a href="schema/generator/index.html"><strong aria-hidden="true">3.3.</strong> Code generation</a></li><li><ol class="section"><li class="chapter-item expanded "><div><strong aria-hidden="true">3.3.1.</strong> API documentation</div></li><li class="chapter-item expanded "><a href="schema/generator/template.html"><strong aria-hidden="true">3.3.2.</strong> Config templates</a></li><li class="chapter-item expanded "><a href="schema/generator/json-schema.html"><strong aria-hidden="true">3.3.3.</strong> JSON schemas</a></li><li class="chapter-item expanded "><a href="schema/generator/typescript.html"><strong aria-hidden="true">3.3.4.</strong> TypeScript types</a></li></ol></li></ol></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
