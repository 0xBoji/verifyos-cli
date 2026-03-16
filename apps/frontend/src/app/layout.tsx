import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import GoogleAnalytics from "../components/GoogleAnalytics";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "verifyOS",
  description:
    "Scan iOS bundles for App Store review risks with agent-ready reports.",
  icons: {
    icon: "/logo/verifyOS_web_128x_round.png",
    shortcut: "/logo/verifyOS_web_128x_round.png",
    apple: "/logo/verifyOS_web_128x_round.png",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased`}
      >
        <GoogleAnalytics
          measurementId={process.env.NEXT_PUBLIC_GA_ID ?? ""}
        />
        {children}
      </body>
    </html>
  );
}
