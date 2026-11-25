/** @type {import('tailwindcss').Config} */
export default {
content: ['./src/**/*.{html,js,svelte,ts}'],
theme: {
extend: {
colors: {
ink: {
950: '#0b0912',
900: '#100d1b',
850: '#171328',
800: '#1e1935',
700: '#2b2448'
},
line: '#342c55',
chalk: '#f3f0ff',
dim: '#a8a0c8',
amber: { DEFAULT: '#ffb63d', 600: '#eda01f' },
mint: '#45d6a4',
coral: '#ff7373',
azure: '#74a8ff',
lilac: '#b18cff'
},
fontFamily: {
display: ['Unbounded', 'system-ui', 'Segoe UI', 'sans-serif'],
sans: ['"Golos Text"', 'system-ui', 'Segoe UI', 'sans-serif']
},
borderRadius: { xl2: '20px' },
keyframes: {
rise: {
from: { opacity: '0', transform: 'translateY(10px)' },
to: { opacity: '1', transform: 'translateY(0)' }
},
pulseSoft: {
'0%, 100%': { opacity: '1' },
'50%': { opacity: '.45' }
}
},
animation: {
rise: 'rise .45s cubic-bezier(.2,.8,.2,1) both',
'pulse-soft': 'pulseSoft 1.6s ease-in-out infinite'
}
}
},
plugins: []
};
