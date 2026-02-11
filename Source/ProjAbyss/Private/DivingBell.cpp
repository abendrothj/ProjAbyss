#include "DivingBell.h"
#include "Components/StaticMeshComponent.h"
#include "Components/SpotLightComponent.h"
#include "Components/BoxComponent.h"

ADivingBell::ADivingBell()
{
	PrimaryActorTick.bCanEverTick = true;

	// Root: Bell Mesh with physics
	BellMesh = CreateDefaultSubobject<UStaticMeshComponent>(TEXT("BellMesh"));
	RootComponent = BellMesh;

	BellMesh->SetSimulatePhysics(true);
	BellMesh->SetMassOverrideInKg(NAME_None, 2000.0f);
	BellMesh->SetLinearDamping(0.8f);  // High linear damping for water resistance
	BellMesh->SetAngularDamping(1.0f);

	// Spotlight pointing down (-90 pitch)
	BellLight = CreateDefaultSubobject<USpotLightComponent>(TEXT("BellLight"));
	BellLight->SetupAttachment(BellMesh);
	BellLight->SetRelativeRotation(FRotator(-90.0f, 0.0f, 0.0f));
	BellLight->SetIntensity(20000.0f);
	BellLight->SetCastShadows(true);

	// Oxygen volume (box for future player detection)
	OxygenVolume = CreateDefaultSubobject<UBoxComponent>(TEXT("OxygenVolume"));
	OxygenVolume->SetupAttachment(BellMesh);
}

void ADivingBell::BeginPlay()
{
	Super::BeginPlay();
	CurrentOxygen = MaxOxygen;
}

void ADivingBell::Tick(float DeltaTime)
{
	Super::Tick(DeltaTime);

	// Drain oxygen when below water level (Z = 0)
	if (GetActorLocation().Z < 0.0f)
	{
		CurrentOxygen = FMath::Max(0.0f, CurrentOxygen - OxygenDrainRate * DeltaTime);
	}
}
