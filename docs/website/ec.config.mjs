import { pluginCollapsibleSections } from '@expressive-code/plugin-collapsible-sections'

/** @type {import('@astrojs/starlight/expressive-code').StarlightExpressiveCodeOptions} */
export default {
  plugins: [pluginCollapsibleSections()],
  defaultProps: {
    // Collapses at top on top & middle lines,
    // Collapses at bottom on bottom line.
    collapseStyle: 'collapsible-auto',
  },
}
