This is the verifyOS web frontend (Next.js). It provides a clean UI to upload an `.ipa` or `.app` and shows scan results from the backend API.

## Getting Started

Install dependencies and start the dev server:

```bash
cd apps/frontend
npm install
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) with your browser to see the result.

To run the backend in another terminal:

```bash
cargo run --manifest-path apps/backend/Cargo.toml
```

The UI expects the backend at `http://127.0.0.1:7070` unless otherwise configured.

Override the backend URL with:

```bash
export NEXT_PUBLIC_BACKEND_URL=https://api.verifyos.com
```

The Google login button redirects to `${NEXT_PUBLIC_BACKEND_URL}/api/v1/auth/google`.

Enable Google Analytics by setting:

```bash
export NEXT_PUBLIC_GA_ID=G-XXXXXXXXXX
```

## Development Notes

- Main UI: `app/page.tsx`
- API integration: `app/api` or client hooks when added

## Learn More

To learn more about Next.js, take a look at the following resources:

- [Next.js Documentation](https://nextjs.org/docs) - learn about Next.js features and API.
- [Learn Next.js](https://nextjs.org/learn) - an interactive Next.js tutorial.
