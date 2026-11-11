import type { Metadata } from "next";
import "@/styles/globals.css";

export const metadata: Metadata = {
  title: "TracePilot",
  description: "Agent 运行时可观测平台",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="zh">
      <body className="bg-gray-50 text-gray-900">{children}</body>
    </html>
  );
}
