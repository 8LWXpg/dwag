Push-Location $PSScriptRoot

$name = 'dwag'
$targetArch = @('win-x64', 'win-arm64')
$tempDir = "./out/$name"

$tag = Read-Host 'New tag'
Set-Content -Path .\dwag.csproj -NoNewline ((Get-Content .\dwag.csproj -Raw) -replace '<Version>\d+\.\d+\.\d+\.0</Version>', "<Version>$tag.0</Version>")
git tag v$tag
git push --tags

mkdir out -ErrorAction Ignore > $null
Remove-Item ./out/*.zip -Recurse -Force -ErrorAction Ignore
foreach ($arch in $targetArch) {
    $releasePath = "./bin/Release/net9.0-windows/$arch/publish/$name.exe"

    dotnet publish -r $arch -c Release

    Remove-Item "$tempDir/*" -Recurse -Force -ErrorAction Ignore
    mkdir "$tempDir" -ErrorAction Ignore
    Copy-Item $releasePath, ../README.md, ../CHANGELOG.md $tempDir -Recurse -Force
    Compress-Archive "$tempDir" "./out/$name-$version-$arch.zip" -Force
}

gh release create v$tag --notes (parse-changelog ..\CHANGELOG.md) (Get-ChildItem ./out/*.zip)

Pop-Location