#include "MasterShip.h"

AMasterShip::AMasterShip()
{
    PrimaryActorTick.bCanEverTick = true;

    // 1. Create the Hull Mesh
    HullMesh = CreateDefaultSubobject<UStaticMeshComponent>(TEXT("HullMesh"));
    RootComponent = HullMesh;

    // Physics Setup (Important!)
    HullMesh->SetSimulatePhysics(true);
    HullMesh->SetMassOverrideInKg(NAME_None, 1000.0f, true); // 1000kg default mass
    HullMesh->SetLinearDamping(0.1f);  // Air resistance
    HullMesh->SetAngularDamping(0.5f); // Rotational air resistance

    // 2. Create the 4 Pontoons
    // We place them in a square around the center.
    // In the Editor, you can move these components to fit your specific boat model.
    PontoonFL = CreateDefaultSubobject<USceneComponent>(TEXT("PontoonFL"));
    PontoonFL->SetupAttachment(RootComponent);
    PontoonFL->SetRelativeLocation(FVector(200.f, -150.f, 0.f));

    PontoonFR = CreateDefaultSubobject<USceneComponent>(TEXT("PontoonFR"));
    PontoonFR->SetupAttachment(RootComponent);
    PontoonFR->SetRelativeLocation(FVector(200.f, 150.f, 0.f));

    PontoonBL = CreateDefaultSubobject<USceneComponent>(TEXT("PontoonBL"));
    PontoonBL->SetupAttachment(RootComponent);
    PontoonBL->SetRelativeLocation(FVector(-200.f, -150.f, 0.f));

    PontoonBR = CreateDefaultSubobject<USceneComponent>(TEXT("PontoonBR"));
    PontoonBR->SetupAttachment(RootComponent);
    PontoonBR->SetRelativeLocation(FVector(-200.f, 150.f, 0.f));
}

void AMasterShip::BeginPlay()
{
    Super::BeginPlay();
}

void AMasterShip::Tick(float DeltaTime)
{
    Super::Tick(DeltaTime);

    // 1. Update the Solver's internal time
    OceanSolver.Time = GetWorld()->GetTimeSeconds();

    // 2. Loop through our 4 Pontoons
    USceneComponent* Pontoons[] = { PontoonFL, PontoonFR, PontoonBL, PontoonBR };

    bool bIsUnderwater = false;

    for (USceneComponent* Pontoon : Pontoons)
    {
        FVector PontoonLoc = Pontoon->GetComponentLocation();

        // ASK THE MATH: "How high is the wave here?"
        float WaveHeight = OceanSolver.GetWaveHeightAt(PontoonLoc);

        // CHECK: Is the pontoon below that wave?
        if (PontoonLoc.Z < WaveHeight)
        {
            bIsUnderwater = true;
            float Depth = WaveHeight - PontoonLoc.Z;

            // PHYSICS: Apply Upward Force
            // Formula: Force = Depth * Multiplier
            FVector UpwardForce = FVector(0, 0, Depth * FloatForce);
            HullMesh->AddForceAtLocation(UpwardForce, PontoonLoc);

            // DRAG: Apply Water Resistance (stops it from bouncing forever)
            // Get velocity specifically at this corner
            FVector VelocityAtPoint = HullMesh->GetPhysicsLinearVelocityAtPoint(PontoonLoc);
            // Push opposite to velocity
            HullMesh->AddForceAtLocation(-VelocityAtPoint * WaterDrag, PontoonLoc);
        }
    }
}