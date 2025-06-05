Push-Location $PSScriptRoot

$name = 'dwag'
$targetArch = @('win-x64', 'win-arm64')

$tag = Read-Host 'New tag'
Set-Content -Path .\dwag.csproj -NoNewline ((Get-Content .\dwag.csproj -Raw) -replace '<Version>\d+\.\d+\.\d+\.0</Version>', "<Version>$tag.0</Version>")
git tag v$tag

mkdir out -ErrorAction Ignore > $null
Remove-Item ./out/*.zip -Recurse -Force -ErrorAction Ignore
foreach ($arch in $targetArch) {
    $releasePath = "./bin/Release/net9.0-windows/$arch/publish/$name.exe"

    dotnet publish -r $arch -c Release
    Copy-Item $releasePath "./out/$name-$arch.exe"
}

gh release create v$tag --notes (parse-changelog ..\CHANGELOG.md) (Get-ChildItem ./out/*.exe)

Pop-Location