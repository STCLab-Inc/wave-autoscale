import dayjs from 'dayjs';
import {
  Bar,
  BarChart,
  CartesianAxis,
  CartesianGrid,
  ComposedChart,
  Label,
  Legend,
  ReferenceArea,
  ReferenceDot,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts';

export default function AutoscalingTimelineChart({
  data,
  yDataKey = 'value',
  xDataKey = 'date',
  zDataKey = 'status',
  xFrom,
  xTo,
  syncId = 'syncId',
}: {
  data: any;
  yDataKey: string;
  xDataKey: string;
  zDataKey: string;
  xFrom: number;
  xTo: number;
  syncId?: string;
}) {
  // console.log({
  //   x: data?.[0]?.[xDataKey],
  //   y: data?.[0]?.[yDataKey],
  //   xFrom,
  //   xTo,
  // });

  return (
    <ResponsiveContainer width="100%" height="100%">
      <ComposedChart
        data={data}
        margin={{
          top: 30,
          right: 30,
          left: 30,
          bottom: 0,
        }}
        syncId={syncId}
      >
        <defs>
          <linearGradient id="areaColor" x1="0" y1="0" x2="0" y2="1">
            <stop offset="60%" stopColor="#3354FF" stopOpacity={0.5} />
            <stop offset="100%" stopColor="#FFFFFF" stopOpacity={0.5} />
          </linearGradient>
        </defs>
        <XAxis
          dataKey={xDataKey}
          domain={[xFrom, xTo]}
          type="number"
          tickFormatter={(value) =>
            dayjs.unix(value).format('YYYY-MM-DD hh:mm')
          }
          // tickCount={8}
          tickSize={0}
          tickMargin={10}
          style={{
            fontSize: '0.75rem',
            color: '#EEEEEE',
          }}
        />
        <YAxis
          dataKey={yDataKey}
          domain={[-1, 0, 1, 2, 3, 4, 5]}
          // tickCount={}
          tickSize={0}
          style={{
            fontSize: '0',
          }}
        />
        <Tooltip />
        <Legend />
        <CartesianGrid strokeDasharray="3 3" />
        {data?.map((item: any, index: number) => {
          return (
            <ReferenceDot
              key={index}
              r={5}
              fill={item[zDataKey] ? '#3354FFAA' : '#FF0000AA'}
              stroke="#eeeeee"
              strokeWidth={1}
              x={item[xDataKey]}
              y={item[yDataKey]}
            />
          );
        })}
      </ComposedChart>
    </ResponsiveContainer>
  );
}
