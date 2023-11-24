export default async function ScalingPlansLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <div className="flex h-full w-full flex-row">{children}</div>;
}
