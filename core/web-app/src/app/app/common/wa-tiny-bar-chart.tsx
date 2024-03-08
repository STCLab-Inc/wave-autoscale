import {
  Bar,
  BarChart,
  CartesianAxis,
  Label,
  ResponsiveContainer,
  XAxis,
} from 'recharts';

export default function WATinyBarChart({
  data,
  yDataKey = 'value',
  xDataKey = 'date',
}: {
  data: any;
  yDataKey: string;
  xDataKey: string;
}) {
  return (
    <ResponsiveContainer width="100%" height="100%">
      <BarChart data={data}>
        <defs>
          <linearGradient id="areaColor" x1="0" y1="0" x2="0" y2="1">
            <stop offset="60%" stopColor="#3354FF" stopOpacity={0.5} />
            <stop offset="100%" stopColor="#FFFFFF" stopOpacity={0.5} />
          </linearGradient>
        </defs>
        <Bar dataKey={yDataKey} fill="url(#areaColor)" />
        <XAxis
          dataKey={xDataKey}
          tickLine={false}
          tick={false}
          height={1}
          axisLine={{
            stroke: '#EEEEEE',
          }}
        >
          <Label value="" />
        </XAxis>
      </BarChart>
    </ResponsiveContainer>
  );
}
