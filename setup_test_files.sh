#!/usr/bin/env bash

test_versions=("2022.1.0a13" "2021.1.28f1" "2021.2.2f1" "2020.1.17f1" "2020.2.7f1" "2020.3.22f1" "2019.1.14f1" "2019.2.21f1" "2019.3.15f1" "2019.4.32f1" "2018.4.36f1" "2018.3.14f1" "2018.2.21f1" "2018.1.9f2" "2017.4.40f1" "2017.3.1f1" "2017.2.5f1" "2017.1.5f1")
platforms=("mac")

base_dir="./commands/uvm-generate-modules-json/tests/fixures"
uvm="./target/debug/uvm"

local_platform='unknown'
unamestr=`uname`
if [[ "$unamestr" == 'Linux' ]]; then
   local_platform='linux'
   rust_platfoem='linux'
   hub_install_dir="${HOME}/Unity/Hub/Editor"
elif [[ "$unamestr" == 'Darwin' ]]; then
   local_platform='mac'
   rust_platform='macos'
   hub_install_dir="/Applications/Unity/Hub/Editor"
fi

MOD_FILE_TEMPLATE=$(cat << END
pub mod modules {
    //MODULE_JSON
}

pub mod manifests {
    //MANIFEST_INI
}
END
)

for platform in "${platforms[@]}"
do

    platform_dir="${base_dir}/${platform}"
    test_setup_text="${platform_dir}/test_setup.txt"

    echo "#[cfg(target_os = \"${rust_platform}\")]" > $test_setup_text
    echo "generate_modules_json![" >> $test_setup_text

    for version in "${test_versions[@]}"
    do
        echo "setup test fixures for ${version}"
        echo "----------------------------------"
        echo ""

        components=(${version//./ })
        major="${components[0]}"
        minor="${components[1]}"

        case "${components[2]}" in
        *f*)
            patch_components=(${components[2]//f/ })
            release_type="f"
            ;;
        *a*)
            patch_components=(${components[2]//a/ })
            release_type="a"
            ;;
        esac

        patch=${patch_components[0]}
        revision=${patch_components[1]}

        output_dir="${platform_dir}/v${major}"

        echo "major: ${major}"
        echo "minor: ${minor}"
        echo "patch: ${patch}"
        echo "release type: ${release_type}"
        echo "revision: ${revision}"
        echo "output_dir: ${output_dir}"

        if [ ! -d "$output_dir" ]; then
          echo "create output dir ${output_dir}"
          mkdir -p "${output_dir}"        
        fi

        ./target/debug/uvm download-manifest -o "${output_dir}" -n "${version}_manifest.ini" -p "${rust_platform}" -f -v "${version}"
        if [ "$platform" = "${local_platform}" ]; then
            module_src="${hub_install_dir}/${version}/modules.json"
            module_dst="${output_dir}/${version}_modules.json"
            echo "copy modules json from ${module_src} to ${module_dst}"

            if [ ! -f "$module_dst" ] && [ -f "$module_src" ]
            then
                echo "copy modules json"
                cp "${module_src}" "${module_dst}"
            fi

            echo "Generate Test mod file"
            modfile="${output_dir}/mod.rs"
            if [ ! -f "${modfile}" ]; then
                echo "${MOD_FILE_TEMPLATE}" >> $modfile
            fi

            module_def="pub const UNITY_${major}_${minor}_${patch}_${release_type^^}_${revision}:\&str = include_str!(\"${version}_modules.json\");"
            if ! grep -q -F "    ${module_def}" "${modfile}"; then
                echo "add ${version} module fixum to test setup"
                sed -i -e "s/\/\/MODULE_JSON/\/\/MODULE_JSON\n    ${module_def}/i" $modfile
            fi
            manifest_def="pub const UNITY_${major}_${minor}_${patch}_${release_type^^}_${revision}:\&str = include_str!(\"${version}_manifest.ini\");"
            if ! grep -q -F "    ${manifest_def}" "${modfile}"; then
                echo "add ${version} module fixum to test setup"
                sed -i -e "s/\/\/MANIFEST_INI/\/\/MANIFEST_INI\n    ${manifest_def}/i" $modfile
            fi
            
            echo "    generates_modules_${major}_${minor}, Version::${release_type}(${major}, ${minor}, ${patch}, ${revision}), UNITY_${major}_${minor}_${patch}_${release_type^^}_${revision}," >> $test_setup_text
        fi

        echo "Done"
        echo ""
    done
    echo "];" >> $test_setup_text
done
