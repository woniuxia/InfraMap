function quoteFile(file) {
  return JSON.stringify(file);
}

function joinFiles(files) {
  return files.map(quoteFile).join(" ");
}

export default {
  "*.{ts,vue}": (files) => {
    const quotedFiles = joinFiles(files);

    return [`pnpm exec eslint --fix ${quotedFiles}`, `pnpm exec prettier --write ${quotedFiles}`];
  },
  "*.{js,cjs,mjs,json,md,scss,css,html,yml,yaml}": (files) => {
    const quotedFiles = joinFiles(files);

    return `pnpm exec prettier --write ${quotedFiles}`;
  },
};
