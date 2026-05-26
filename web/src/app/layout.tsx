import type { Metadata } from 'next';
import './globals.css';

export const metadata: Metadata = {
  title: 'atom — hydrogen orbital visualizer',
  description: 'Real-time ray-marched hydrogen |ψ_nlm|² in the browser, via WASM + WebGL2.',
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
