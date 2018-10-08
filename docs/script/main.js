const lngs = {
    'zh-CN': "中文",
    'en-US': 'English'
}
var initDocsify = function () {
    var configs = [common, customization]
    var docsify = {}
    configs.forEach(function (conf) {
        _.assign(docsify, conf)
    })
    loadTags()

    let ver, lng

    const search = window.location.hash.split("?").pop()
    if (search && search !== '#/' && URLSearchParams) {
        const urlP = new URLSearchParams(search)
        ver = urlP.get("version")
        lng = urlP.get("language")
    }
    ver = ver || window.localStorage.getItem("version") || 'develop'
    lng = lng || window.localStorage.getItem('lng') || 'zh-CN'

    const vTag = document.getElementById('tag_version')
    vTag.textContent = ver === 'develop' ? 'Latest' : ver
    const lTag = document.getElementById('tag_language')
    lTag.textContent = lngs[lng]
    docsify.basePath = baseUrlGen(ver, lng)
    window.$docsify = docsify
}

var main = function () {
    initDocsify()
}

main()
