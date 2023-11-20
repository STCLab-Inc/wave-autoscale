import Link from 'next/link';

export default function AppPage() {
  return (
    <div className="h-full bg-gradient-to-b from-blue-700 via-blue-500 to-blue-900 text-white">
      {/* Header */}
      <header className="bg-blue-800 px-10 py-4">
        <div className="container mx-auto flex flex-col items-center justify-between sm:flex-row">
          <Link href="https://www.waveautoscale.com/">
            <button className="text-2xl font-extrabold hover:text-blue-300">
              Wave Autoscale
            </button>
          </Link>
          <ul className="flex space-x-4 text-xl">
            <li>
              <Link href="https://www.waveautoscale.com/about/introduction">
                <button className="hover:text-blue-300">Introduction</button>
              </Link>
            </li>
            <li>
              <Link href="https://www.waveautoscale.com/about/key-features">
                <button className="hover:text-blue-300">Features</button>
              </Link>
            </li>
            <li>
              <Link href="https://www.waveautoscale.com/about/principles">
                <button className="hover:text-blue-300">Principles</button>
              </Link>
            </li>
            <li>
              <Link href="https://www.waveautoscale.com/blog">
                <button className="hover:text-blue-300">Blog</button>
              </Link>
            </li>
          </ul>
        </div>
      </header>

      {/* Hero Section */}
      <section className="container mx-auto px-10 py-12 text-center">
        <h1 className="mb-6 text-2xl font-extrabold text-white sm:text-3xl">
          Autoscale Your Applications with Confidence
        </h1>
        <p className="mb-10 text-lg sm:text-2xl">
          Experience high availability, scalability, and performance like never
          before.
        </p>
        <Link href="https://github.com/stclab-inc/wave-autoscale">
          <button className="rounded-full bg-blue-500 px-8 py-3 text-xl font-semibold text-white transition duration-300 hover:bg-blue-600 sm:py-4">
            Get Started
          </button>
        </Link>
      </section>

      {/* Features Section */}
      <section className="bg-blue-800 px-10 py-10 sm:py-16 lg:py-20">
        <div className="container mx-auto text-center">
          <h2 className="mb-12 text-2xl font-extrabold text-white sm:text-3xl lg:text-3xl">
            Discover Our Features
          </h2>
          <div className="grid grid-cols-1 gap-6 sm:gap-12 md:grid-cols-2 lg:grid-cols-3">
            {/* Feature 1 */}
            <div className="rounded-lg bg-blue-900 p-6 sm:p-8">
              <h3 className="mb-3 text-2xl font-semibold sm:text-2xl">
                Auto Scaling
              </h3>
              <p className="text-lg sm:text-xl">
                Effortlessly adjust resources based on demand.
              </p>
            </div>
            {/* Feature 2 */}
            <div className="rounded-lg bg-blue-900 p-6 sm:p-8">
              <h3 className="mb-3 text-2xl font-semibold sm:text-2xl">
                High Availability
              </h3>
              <p className="text-lg sm:text-xl">
                Ensure your applications are always available.
              </p>
            </div>
            {/* Feature 3 */}
            <div className="rounded-lg bg-blue-900 p-6 sm:p-8">
              <h3 className="mb-3 text-2xl font-semibold sm:text-2xl">
                Scalability
              </h3>
              <p className="text-lg sm:text-xl">
                Scale your applications effortlessly as you grow.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="bg-blue-900 py-6 text-center sm:py-8">
        <div className="container mx-auto">
          <p className="text-lg">
            &copy; 2023 STCLab Inc. All rights reserved.
          </p>
        </div>
      </footer>
    </div>
  );
}
