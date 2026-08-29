import { NextResponse } from "next/server";
import { readFile } from "fs/promises";

export const dynamic = "force-dynamic";

export const GET = async () => {
  const path = process.env.DESKTOP_BINARY_PATH ?? "/shared/client.dmg";

  try {
    const file = await readFile(path);
    return new NextResponse(file, {
      headers: {
        "Content-Type": "application/octet-stream",
        "Content-Disposition": 'attachment; filename="vigil.dmg"',
      },
    });
  } catch {
    return NextResponse.json(
      { error: "desktop binary not available yet" },
      { status: 404 },
    );
  }
};
