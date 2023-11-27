export default async function InflowLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <div className="flex h-full w-full flex-row">{children}</div>;
}
