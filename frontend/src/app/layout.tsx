import type { Metadata } from "next";
import Link from "next/link";
import "@/styles/globals.css";

export const metadata: Metadata = {
  title: "TracePilot",
  description: "Agent 运行时可观测平台",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="zh">
      <body className="bg-gray-50 text-gray-900">
        <nav className="border-b bg-white">
          <div className="mx-auto flex max-w-5xl items-center gap-6 px-8 py-3">
            <Link href="/" className="font-semibold">
              TracePilot
            </Link>
            <div className="flex gap-4 text-sm">
              <Link href="/" className="text-blue-600 hover:underline">
                Traces
              </Link>
              <Link href="/monitor" className="text-blue-600 hover:underline">
                监控
              </Link>
              <Link href="/analytics" className="text-blue-600 hover:underline">
                分析
              </Link>
              <Link href="/recovery" className="text-blue-600 hover:underline">
                恢复
              </Link>
              <Link href="/alerts" className="text-blue-600 hover:underline">
                告警
              </Link>
            </div>
          </div>
        </nav>
        {children}
      </body>
    </html>
  );
}
