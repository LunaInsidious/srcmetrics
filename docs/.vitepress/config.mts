import { readdirSync, readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { defineConfig, type DefaultTheme } from 'vitepress'

// User documentation (ADR-0023). Developer documents live in design/ and are not published.

const metricsDir = fileURLToPath(new URL('../metrics', import.meta.url))

// The metric reference pages are generated from the code (docs/metrics/). List them in the order
// the generated overview links to them, which is the definition order.
function metricPages(): DefaultTheme.SidebarItem[] {
  const overview = readFileSync(`${metricsDir}/index.md`, 'utf8')
  const groups = [...new Set([...overview.matchAll(/\]\(\.\/([a-z_]+)#/g)].map((m) => m[1]))]
  const files = readdirSync(metricsDir).filter((f) => f !== 'index.md')
  if (files.length !== groups.length) {
    throw new Error('docs/metrics/ is out of date; regenerate with UPDATE_DOCS=1 cargo test -p srcmetrics --test docs')
  }
  return groups.map((group) => {
    const title = readFileSync(`${metricsDir}/${group}.md`, 'utf8').match(/^# (.+)$/m)
    if (!title) throw new Error(`docs/metrics/${group}.md has no title`)
    return { text: title[1], link: `/metrics/${group}` }
  })
}

function sidebar(prefix: string, labels: Record<string, string>): DefaultTheme.Sidebar {
  const guide = (page: string) => `${prefix}/guide/${page}`
  return [
    {
      text: labels.guide,
      items: [
        { text: labels.gettingStarted, link: guide('getting-started') },
        { text: labels.cli, link: guide('cli') },
        { text: labels.output, link: guide('output') },
        { text: labels.library, link: guide('library') },
        { text: labels.model, link: guide('model') },
        { text: labels.httpApi, link: guide('http-api') },
        { text: labels.languages, link: guide('languages') },
      ],
    },
    {
      text: labels.metrics,
      items: [{ text: labels.overview, link: '/metrics/' }, ...metricPages()],
    },
  ]
}

export default defineConfig({
  title: 'srcmetrics',
  description: 'Language-independent source code metrics',
  base: '/srcmetrics/',
  cleanUrls: true,
  lastUpdated: true,
  themeConfig: {
    search: { provider: 'local' },
    socialLinks: [{ icon: 'github', link: 'https://github.com/LunaInsidious/srcmetrics' }],
  },
  locales: {
    root: {
      label: 'English',
      lang: 'en',
      themeConfig: {
        nav: [
          { text: 'Guide', link: '/guide/getting-started' },
          { text: 'Metrics', link: '/metrics/' },
        ],
        sidebar: sidebar('', {
          guide: 'Guide',
          gettingStarted: 'Getting started',
          cli: 'Command-line reference',
          output: 'Output format',
          library: 'Library',
          model: 'Readability model',
          httpApi: 'HTTP API',
          languages: 'Languages and limitations',
          metrics: 'Metric reference',
          overview: 'Overview',
        }),
      },
    },
    ja: {
      label: '日本語',
      lang: 'ja',
      link: '/ja/',
      themeConfig: {
        nav: [
          { text: 'ガイド', link: '/ja/guide/getting-started' },
          { text: 'メトリクス定義', link: '/metrics/' },
        ],
        sidebar: sidebar('/ja', {
          guide: 'ガイド',
          gettingStarted: 'はじめに',
          cli: 'コマンドリファレンス',
          output: '出力形式',
          library: 'ライブラリ',
          model: '可読性モデル',
          httpApi: 'HTTP API',
          languages: '対応言語と制約',
          metrics: 'メトリクス定義（英語）',
          overview: '一覧',
        }),
        outline: { label: '目次' },
        docFooter: { prev: '前のページ', next: '次のページ' },
        lastUpdated: { text: '最終更新' },
      },
    },
  },
})
