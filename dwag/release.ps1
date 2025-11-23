Push-Location $PSScriptRoot

$name = 'dwag'
$targetArch = @('win-x64', 'win-arm64')
$tempDir = "./out/$name"

$tag = Read-Host 'New tag'
Set-Content -Path .\dwag.csproj -NoNewline ((Get-Content .\dwag.csproj -Raw) -replace '<Version>\d+\.\d+\.\d+\</Version>', "<Version>$tag</Version>")
git commit -am 'bump'
git tag v$tag
git push
git push --tags

mkdir out -ErrorAction Ignore > $null
Remove-Item ./out/*.zip -Recurse -Force -ErrorAction Ignore
foreach ($arch in $targetArch) {
    $publishDir = dotnet publish -r $arch -c Release -getProperty:PublishDir
    $releasePath = "$publishDir$name.exe"

    Remove-Item "$tempDir/*" -Recurse -Force -ErrorAction Ignore
    mkdir "$tempDir" -ErrorAction Ignore
    Copy-Item $releasePath, ../README.md, ../CHANGELOG.md $tempDir -Recurse -Force
    Compress-Archive "$tempDir" "./out/$name-$tag-$arch.zip" -Force
}

gh release create v$tag -n ((parse-changelog ..\CHANGELOG.md) -join "`n") (Get-ChildItem ./out/*.zip)

Pop-Location