# S04: v2 TAR/XZ managed package 통합 계약

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R05, R11
- 담당/수정 범위: metrics_recovery — 이 문서만; root shared schema/API 및 후속 파일 담당 조정
- 선행 작업: 실제 고정 FFmpeg TAR/XZ 세component 추출과 원source SHA/비용 측정; 기존 v1 ZIP 설치·inventory·removal

## 사용자 결과

공급에서 고정한 TAR/XZ FFmpeg·ffprobe·LICENSE를 한 관리 세트로 설치·검증·선택·제거한다. 기존 v1 ZIP/Deno 저장 자료를 읽고 같은 정책으로 유지한다. 임의 manifest 업로드 GUI를 추가하지 않는다.

## 최소 명시 v2 schema

ArchivePackage.version1은 layout 없음으로 기존 flatZIP/expanded256MiB를 그대로 해석한다. 추가 serde(default,skipNone) layout:Option<ArchiveLayout>을 root가 조정한다. version1+layoutSome은 거절하고 version2는 명시 TarXz{root,mappings,limits}만 허용한다. root는 안전한 단일 ASCII path component, mappings는 member 원상대경로→flat component name의1:1 관계이며 모든 package component와 정확히 일치해야 한다. case alias/duplicates/parent/absolute/backslash/control/unknownfield/없는component/다른root를 거절한다. 기존 flat names/reserved Windows device/두metadata 충돌 정책은 유지한다. mapping이 component digest/size를 별도 중복 소유하지 않고 기존 components가 유일 hash authority다.

TarLimits는 schema에 공급 고정 상한을 담거나 enum별 정책으로 고정한다. TAR entries≤4096/전체선언regular bytes≤768MiB/검증stream≤800MiB, 선택 설치총합≤384MiB를 분리한다. 단일component 상한384MiB만으로 전체선택합 상한을 대신하지 않는다. v1 ZIP의32entries/전체256MiB는 그대로다. archive compressed는≤256MiB, sourceURL/license/manifest64KiB도 유지한다. v2 parser의 unknownfields/version/layout mismatch는 preserve/거절이며 기존 JSON 저장 order 변경을 hashproof 재생성으로 조용히 대체하지 않는다.

## 설치·capacity·staging 공유 API

archive_install from_file adapter의 component loop만 layout에 따라 ZIP 또는 extract_verified_tar_xz로 dispatch한다. source 원File/nativeID, 한 root install lock, 동일 absolute120초, pre/post archive hash, 원 staging evidence/create-only publish/최종fullset hash를 유지한다. 새 generic decoder/external tar process/전체 memoryread는 없다. mapping 변환은 component8개 bounded Vec이고 출력은 같은 flat managed directory다. inventory/selection/removal은 포맷과 무관하게 exact installedflatcomponent+두manifest를 검증하므로 새 삭제 fallback을 만들지 않는다.

768MiB는 폐기하면서 검증하는 TAR 전체 선언 상한이며 저장 예약이 아니다. 실제 선택271.65MiB와 sourcearchive131.48MiB를 합친403.13MiB(422710799B)+metadata/journal이 staging512MiB 아래에 들어가는지 checked-add로 예약한다. archive_download의 archive+selected+2manifest 선예약 및 internal 현재 원archive stage accounting+selected 재확인을 유지한다. 최종selected capacity와 metadata-only inventory128/entries2048도 유지한다. partial/unknown journal을 임의 제거하지 않는다.

## 비용·기한 해석

현재 singlecomponent3회 stream 추출 실제 약35초는 고정Linux 후보의 측정이다. installer120초 안에 기존3회decode/hash+copy/publication이 들어갈 가능성은 있지만 전체 설치 actualdeadline/RSS 확인 전 적합 완료로 표시하지 않는다. 동일 source 원SHA를각 stream 전에 검증하고 마지막sourcehash를 유지한다. loop별 deadline을 새로120초로 재생성하지 않는다. 느린disk·Windows·악성압축/cancel responsiveness는 별도실증이다. 최적화가 필요하면 multioutput singlepass는 후속 공유검증이며 현재성공을 위해 hash검증을 삭제하지 않는다.

## 수정 파일과 constructors

root archive_manifest.rs 및 tools.rs export를 먼저 확정한다. archive_install.rs layoutdispatch/tests, archive_download.rs reservation/API regression, package_inventory.rs metadata compatibility/tests, package_removal.rs format-independent tests, staging_recovery.rs package schema proof 필요성은 root가 담당 분리한다. 현재 ArchivePackage literals는 archive_manifest.rs, bounded_zip.rs, archive_install.rs, package_inventory.rs, package_removal.rs, deno_version.rs 및 실제 fixture/examples를 rg로 다시 확인하여 v1 layoutNone을 일괄 추가한다. verify_ffmpeg_tar_xz example은 rawAPI라 package constructor가 없지만 실제 v2 install example은 새 typedpin이 필요하다. serde v1 unknownlayout 허용 및 oldJSON→roundtrip regression을 추가한다.

## 실패 인수와 공급 UI

v1 JSON olddecode/reselect/remove, versionlayout mismatch/forgedmapping/duplicatemember/selectedsumover384/declaredTARover768/XZmemorylimit/truncatedCRCtail/nonregular/absoluteparent/symlinkhardlink/partialoutput/hash/DBcheckpoint/publication/DELETEpreflight 실패에서 원active/source/foreign 보존을 확인한다. 실제 고정원archive→inactive v2package→ffmpeg와ffprobe 정확version/nativeposthash→선택→active 제거거절→비활성exactremove를 root native에서 검증한다. 대응source/build recipe/thirdparty notice/trust/3OS gate는 TAR codec 구현으로 해결되지 않는다. 도구 UI는 고정공급·크기·license·지원target/보류이유를 표시하고 공급미채택을 숨기지 않는다.

## 검증과 handoff

schema/API·capacity·staging·inventory·remove 읽기 감사와 이 계약 문서만 작성했다. sourcefreeze를 유지해 Rust/Cargo/프로세스/다운로드를 변경·실행하지 않았다. Proposed 공유계약이며 root schema 조정 후 담당별 실제 구현·통합 검증이 필요하다.

## 통합 담당 구현 계약 확정

root는v1 layoutNone/JSONskipNone와v2 ArchiveLayout::TarXz{root,mappings}를구현기준으로정한다. ArchiveMemberMapping은member와component명만저장하며bytes/SHA는기존components가유일authority다. TARlimits는manifest사용자설정으로늘릴수없도록형식별고정policy로둔다. 기존ZIP256MiB는유지하고v2선택합384MiB를명시한다. immutable실제공급형식·측정설치세트271.65MiB와기존512MiBstage상한에따른가역구현결정이며공급trust/라이선스채택승인이아니다. sourcefreeze종료전독립layoutmodule초안은/tmp에작성하고root가별도schema파일·constructor·dispatch를통합한다. 같은프로젝트파일의동시수정은없다.

선행Linux전체collector는9operationterminalexit0/workspace_checks_complete이며root가source_final현재SHA불일치0확인후소스동결을해제했다. 이전전체PASS와새v2변경을구분한다. root가공유schema·constructor·설치dispatch를담당한다.

root는archive_layout.rs/manifest의serde(default,skipNone),버전별상한,기존6개constructorlayoutNone,ZIP전용decoder의v2거절,archiveinstaller XZdispatch와transfer의archive.tar.xz원checkpoint이름을통합했다. 매핑오류는panic없이typed오류로반환하고압축/component SHA대소문자는decoder계약에맞춰정규화한다. Linux도구70PASS/1ignored·engineall-targetClippyexit0와신규실제package3회귀PASS를확인했다. sharedschema/Cargo는root가조정했고dependency는직전XZsnapshot과같다. latest전체workspace/실제공식설치/OS검증은후속이다.

## 공식 v2 관리 설치 실제 인수

root가 release install_managed_archive 예제로고정FFmpeg v2manifest와원tar.xz를임시managedroot에설치했다. actualexit0/PASS: 선택합284847347B, 비활성유지,전체component검증, 정확세트제거와전후원archive SHA보존. install36682ms/검증제거2641ms/외부전체40.44초로원120초installer기한안에성공했다. [실제결과](../validation/evidence/ffmpeg-v2-managed-install-first/result.json). 실제runtime버전선택/active제거보호는이실험에서실행하지않았고하위primitive합성시험의증거와구분한다. dependency/codec추출성공을trust/license/Windows/macOS/GUI공급완료로확대하지않는다. 신규package3회귀와후속engineall-targetClippy도terminalexit0이며전체workspace는새source에대해후속검증한다.

실제관리예제53581c6c…의전체binary3788824B, /usr/bin/time최대RSS12808KiB/wall37.45초도기록한다. 제품RustInstant설치36682ms와외부Pythonelapsed40.44초는원시관측별로구분하며UI앱증분/정밀종합benchmark로해석하지않는다. 다음정확ffmpeg/ffprobe versionparser초안은/tmp에작성돼root소스통합전이며프로세스실행성공으로표시하지않는다.
